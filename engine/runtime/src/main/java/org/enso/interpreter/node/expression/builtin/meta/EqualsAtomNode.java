package org.enso.interpreter.node.expression.builtin.meta;

import com.oracle.truffle.api.CompilerAsserts;
import com.oracle.truffle.api.CompilerDirectives;
import com.oracle.truffle.api.CompilerDirectives.TruffleBoundary;
import com.oracle.truffle.api.dsl.Cached;
import com.oracle.truffle.api.dsl.Cached.Shared;
import com.oracle.truffle.api.dsl.GenerateUncached;
import com.oracle.truffle.api.dsl.Specialization;
import com.oracle.truffle.api.library.CachedLibrary;
import com.oracle.truffle.api.nodes.ExplodeLoop;
import com.oracle.truffle.api.nodes.Node;
import com.oracle.truffle.api.profiles.ConditionProfile;
import java.util.Arrays;
import org.enso.interpreter.node.callable.InvokeCallableNode.ArgumentsExecutionMode;
import org.enso.interpreter.node.callable.InvokeCallableNode.DefaultsExecutionMode;
import org.enso.interpreter.node.callable.dispatch.InvokeFunctionNode;
import org.enso.interpreter.node.expression.builtin.ordering.CustomComparatorNode;
import org.enso.interpreter.runtime.EnsoContext;
import org.enso.interpreter.runtime.callable.argument.CallArgumentInfo;
import org.enso.interpreter.runtime.callable.atom.Atom;
import org.enso.interpreter.runtime.callable.atom.AtomConstructor;
import org.enso.interpreter.runtime.callable.atom.StructsLibrary;
import org.enso.interpreter.runtime.callable.function.Function;
import org.enso.interpreter.runtime.data.Type;
import org.enso.interpreter.runtime.state.State;

@GenerateUncached
public abstract class EqualsAtomNode extends Node {

  public static EqualsAtomNode build() {
    return EqualsAtomNodeGen.create();
  }

  public abstract boolean execute(Atom left, Atom right);

  static EqualsNode[] createEqualsNodes(int size) {
    EqualsNode[] nodes = new EqualsNode[size];
    Arrays.fill(nodes, EqualsNode.build());
    return nodes;
  }

  @Specialization(
      guards = {
        "selfCtorCached == self.getConstructor()",
        "customComparatorNode.execute(self) == null"
      },
      limit = "10")
  @ExplodeLoop
  boolean equalsAtomsWithDefaultComparator(
      Atom self,
      Atom other,
      @Cached("self.getConstructor()") AtomConstructor selfCtorCached,
      @Cached(value = "selfCtorCached.getFields().length", allowUncached = true)
          int fieldsLenCached,
      @Cached(value = "createEqualsNodes(fieldsLenCached)", allowUncached = true)
          EqualsNode[] fieldEqualsNodes,
      @Shared("customComparatorNode") @Cached CustomComparatorNode customComparatorNode,
      @Cached ConditionProfile constructorsNotEqualProfile,
      @CachedLibrary(limit = "5") StructsLibrary structsLib) {
    if (constructorsNotEqualProfile.profile(self.getConstructor() != other.getConstructor())) {
      return false;
    }
    var selfFields = structsLib.getFields(self);
    var otherFields = structsLib.getFields(other);
    assert selfFields.length == otherFields.length
        : "Constructors are same, atoms should have the same number of fields";

    CompilerAsserts.partialEvaluationConstant(fieldsLenCached);
    for (int i = 0; i < fieldsLenCached; i++) {
      boolean fieldsAreEqual = fieldEqualsNodes[i].execute(selfFields[i], otherFields[i]);
      if (!fieldsAreEqual) {
        return false;
      }
    }
    return true;
  }

  @Specialization(
      guards = {
        "selfCtorCached == self.getConstructor()",
        "cachedComparator != null",
      },
      limit = "10")
  boolean equalsAtomsWithCustomComparator(
      Atom self,
      Atom other,
      @Cached("self.getConstructor()") AtomConstructor selfCtorCached,
      @Shared("customComparatorNode") @Cached CustomComparatorNode customComparatorNode,
      @Cached(value = "customComparatorNode.execute(self)") Type cachedComparator,
      @Cached(value = "findCompareMethod(cachedComparator)", allowUncached = true)
          Function compareFn,
      @Cached(value = "invokeCompareNode(compareFn)") InvokeFunctionNode invokeNode) {
    var otherComparator = customComparatorNode.execute(other);
    if (cachedComparator != otherComparator) {
      return false;
    }
    var ctx = EnsoContext.get(this);
    var args = new Object[] {cachedComparator, self, other};
    var result = invokeNode.execute(compareFn, null, State.create(ctx), args);
    return ctx.getBuiltins().ordering().newEqual() == result;
  }

  @CompilerDirectives.TruffleBoundary
  @Specialization(
      replaces = {"equalsAtomsWithDefaultComparator", "equalsAtomsWithCustomComparator"})
  boolean equalsAtomsUncached(Atom self, Atom other) {
    if (self.getConstructor() != other.getConstructor()) {
      return false;
    }
    Type customComparator = CustomComparatorNode.getUncached().execute(self);
    if (customComparator != null) {
      Function compareFunc = findCompareMethod(customComparator);
      var invokeFuncNode = invokeCompareNode(compareFunc);
      return equalsAtomsWithCustomComparator(
          self,
          other,
          self.getConstructor(),
          CustomComparatorNode.getUncached(),
          customComparator,
          compareFunc,
          invokeFuncNode);
    }
    Object[] selfFields = StructsLibrary.getUncached().getFields(self);
    Object[] otherFields = StructsLibrary.getUncached().getFields(other);
    if (selfFields.length != otherFields.length) {
      return false;
    }
    for (int i = 0; i < selfFields.length; i++) {
      boolean areFieldsSame = EqualsNodeGen.getUncached().execute(selfFields[i], otherFields[i]);
      if (!areFieldsSame) {
        return false;
      }
    }
    return true;
  }

  @TruffleBoundary
  static Function findCompareMethod(Type comparator) {
    var fn = comparator.getDefinitionScope().getMethods().get(comparator).get("compare");
    if (fn == null) {
      throw new AssertionError("No compare function for " + comparator);
    }
    return fn;
  }

  static InvokeFunctionNode invokeCompareNode(Function compareFn) {
    CallArgumentInfo[] argsInfo = new CallArgumentInfo[compareFn.getSchema().getArgumentsCount()];
    for (int i = 0; i < argsInfo.length; i++) {
      var argDef = compareFn.getSchema().getArgumentInfos()[i];
      argsInfo[i] = new CallArgumentInfo(argDef.getName());
    }
    return InvokeFunctionNode.build(
        argsInfo, DefaultsExecutionMode.EXECUTE, ArgumentsExecutionMode.EXECUTE);
  }
}
