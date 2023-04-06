package org.enso.interpreter.node.expression.builtin.bool;

import com.oracle.truffle.api.RootCallTarget;
import com.oracle.truffle.api.Truffle;
import com.oracle.truffle.api.TruffleLogger;
import com.oracle.truffle.api.frame.VirtualFrame;
import com.oracle.truffle.api.nodes.IndirectCallNode;
import com.oracle.truffle.api.nodes.Node;
import com.oracle.truffle.api.profiles.ConditionProfile;
import java.util.logging.Level;
import org.enso.interpreter.EnsoLanguage;
import org.enso.interpreter.dsl.BuiltinMethod;
import org.enso.interpreter.dsl.Suspend;
import org.enso.interpreter.node.BaseNode;
import org.enso.interpreter.node.callable.ExecuteCallNode;
import org.enso.interpreter.node.callable.thunk.ThunkExecutorNode;
import org.enso.interpreter.runtime.EnsoContext;
import org.enso.interpreter.runtime.callable.CallerInfo;
import org.enso.interpreter.runtime.callable.function.Function;
import org.enso.interpreter.runtime.callable.function.Function.ArgumentsHelper;
import org.enso.interpreter.runtime.state.State;

@BuiltinMethod(
    type = "Boolean",
    name = "if_then_else",
    description = "Performs the standard if-then-else control flow operation.")
public class IfThenElseNode extends Node {
  private @Child ThunkExecutorNode leftThunkExecutorNode = ThunkExecutorNode.build();
  private @Child ThunkExecutorNode rightThunkExecutorNode = ThunkExecutorNode.build();
  private final ConditionProfile condProfile = ConditionProfile.createCountingProfile();
  private static final TruffleLogger logger = TruffleLogger.getLogger("enso", "mylogger");

  Object execute(
      VirtualFrame frame,
      State state,
      boolean self,
      @Suspend Object if_true,
      @Suspend Object if_false) {
    if (isNestedIf()) {
      return callReplacedRoot(frame, state, self, if_true, if_false);
    } else {
      if (condProfile.profile(self)) {
        return leftThunkExecutorNode.executeThunk(if_true, state, BaseNode.TailStatus.TAIL_DIRECT);
      } else {
        return rightThunkExecutorNode.executeThunk(
            if_false, state, BaseNode.TailStatus.TAIL_DIRECT);
      }
    }
  }

  private boolean isNestedIf() {
    boolean[] result = new boolean[] {false};
    // TODO: Check only for the first 3 frames or so, not the whole stack
    Truffle.getRuntime()
        .iterateFrames(
            frameInstance -> {
              var callNode = frameInstance.getCallNode();
              if (callNode != null) {
                var rootNode = callNode.getRootNode();
                if (rootNode != null) {
                  if (rootNode.getName().equals("Boolean.if_then_else")) {
                    result[0] = true;
                  }
                }
              }
              return null;
            });
    return result[0];
  }

  private Object callReplacedRoot(
      VirtualFrame frame, State state, boolean self, Object if_true, Object if_false) {
    logger.entering("IfThenElseNode", "callReplacedRoot");
    var rootNode = getRootNode();
    assert rootNode != null;
    assert rootNode.isCloningAllowed();
    Node copiedRootNode = rootNode.copy();
    RootCallTarget callTarget = copiedRootNode.getRootNode().getCallTarget();
    var callNode = IndirectCallNode.getUncached();
    var ctx = EnsoContext.get(this);
    var language = EnsoLanguage.get(this);
    var ifThenElseFunc =
        ctx.getBuiltins().getBuiltinFunction("Boolean", "if_then_else", language, false);
    if (ifThenElseFunc.isEmpty()) {
      throw new IllegalStateException("ifThenElse");
    }
    Function ifThenElseFunction = ifThenElseFunc.get().getFunction();
    Object[] frameArgs = frame.getArguments();
    var callerInfo = Function.ArgumentsHelper.getCallerInfo(frameArgs);
    Object[] arguments =
        ArgumentsHelper.buildArguments(
            ifThenElseFunction, callerInfo, state, new Object[] {self, if_true, if_false});
    // Copied from ExecuteCallNode
    // TODO:
    /**
     * TODO: This causes infinite recursion, as nestedIf() returns true all the time. We have to
     * turn it off somehow.
     */
    Object res = callNode.call(callTarget, arguments);
    logger.exiting("IfThenElseNode", "callReplacedRoot");
    return res;
  }
}
