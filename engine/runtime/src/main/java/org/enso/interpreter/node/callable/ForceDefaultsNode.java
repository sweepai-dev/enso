package org.enso.interpreter.node.callable;

import com.oracle.truffle.api.dsl.Cached;
import com.oracle.truffle.api.dsl.Fallback;
import com.oracle.truffle.api.dsl.NodeChild;
import com.oracle.truffle.api.dsl.Specialization;
import com.oracle.truffle.api.frame.FrameUtil;
import com.oracle.truffle.api.frame.VirtualFrame;
import com.oracle.truffle.api.nodes.NodeInfo;
import org.enso.interpreter.node.ExpressionNode;
import org.enso.interpreter.node.callable.dispatch.InvokeFunctionNode;
import org.enso.interpreter.node.callable.thunk.ForceNodeGen;
import org.enso.interpreter.node.callable.thunk.ThunkExecutorNode;
import org.enso.interpreter.runtime.callable.argument.CallArgumentInfo;
import org.enso.interpreter.runtime.callable.function.Function;
import org.enso.interpreter.runtime.data.Array;
import org.enso.interpreter.runtime.state.Stateful;

/** Node responsible for handling user-requested thunks forcing. */
@NodeInfo(shortName = "Force", description = "Forces execution of a thunk at runtime")
@NodeChild(value = "target", type = ExpressionNode.class)
public abstract class ForceDefaultsNode extends ExpressionNode {

  ForceDefaultsNode() {}

  /**
   * Creates an instance of this node.
   *
   * @param target the expression being forced
   * @return a node representing {@code target} being forced
   */
  public static ForceDefaultsNode build(ExpressionNode target) {
    return ForceDefaultsNodeGen.create(target);
  }

  @Specialization
  Object doFunction(
      VirtualFrame frame,
      Function func,
      @Cached("buildEmptyInvokeFun()") InvokeFunctionNode invokeFunctionNode) {
    Object state = FrameUtil.getObjectSafe(frame, getStateFrameSlot());
    Stateful result = invokeFunctionNode.execute(func, frame, state, new Object[0]);
    frame.setObject(getStateFrameSlot(), result.getState());
    return result.getValue();
  }

  @Fallback
  Object doOther(VirtualFrame frame, Object obj) {
    return obj;
  }

  InvokeFunctionNode buildEmptyInvokeFun() {
    return InvokeFunctionNode.build(
        new CallArgumentInfo[0],
        InvokeCallableNode.DefaultsExecutionMode.EXECUTE,
        InvokeCallableNode.ArgumentsExecutionMode.PRE_EXECUTED);
  }
}
