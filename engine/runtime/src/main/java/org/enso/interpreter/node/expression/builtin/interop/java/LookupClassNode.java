package org.enso.interpreter.node.expression.builtin.interop.java;

import com.oracle.truffle.api.dsl.Cached;
import com.oracle.truffle.api.dsl.Specialization;
import com.oracle.truffle.api.nodes.Node;
import org.enso.interpreter.dsl.BuiltinMethod;
import org.enso.interpreter.node.expression.builtin.text.util.ExpectStringNode;
import org.enso.interpreter.runtime.EnsoContext;

@BuiltinMethod(
    type = "Java",
    name = "lookup_class",
    description = "Looks up a Java symbol.",
    autoRegister = false)
public abstract class LookupClassNode extends Node {
  static LookupClassNode build() {
    return LookupClassNodeGen.create();
  }

  @Specialization
  Object doExecute(Object name, @Cached("build()") ExpectStringNode expectStringNode) {
    var env = EnsoContext.get(this).getEnvironment();
    if ("hosted".equals(System.getenv("ENSO_JAVA"))) {
      return env.lookupHostSymbol(expectStringNode.execute(name));
    }
    System.out.println("Bindings: " + env.getPolyglotBindings());
    Object java = env.importSymbol("java");
    System.out.println("Java: " + java);
    String symbol = expectStringNode.execute(name);
    Object symb = env.importSymbol(symbol);
    System.out.println("symb: " + symb);
    return env.lookupHostSymbol(symbol);
  }

  abstract Object execute(Object name);
}
