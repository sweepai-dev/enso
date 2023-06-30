package org.enso.tools.enso4igv.enso;

import java.io.IOException;
import java.util.Collection;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.function.Consumer;

import java.util.stream.Collectors;
import javax.swing.text.Document;

import org.enso.compiler.context.InlineContext;
import org.enso.interpreter.node.BaseNode.TailStatus;
import org.enso.interpreter.node.ExpressionNode;
import org.enso.interpreter.node.expression.constant.ErrorNode;
import org.enso.interpreter.runtime.library.dispatch.TypesLibrary;
import org.enso.interpreter.runtime.scope.LocalScope;
import org.enso.interpreter.runtime.scope.ModuleScope;
import org.enso.syntax2.Tree;
import org.enso.syntax2.Tree.Ident;
import com.oracle.truffle.api.source.Source;
import org.netbeans.api.editor.mimelookup.MimeRegistration;
import org.netbeans.api.lsp.Completion;
import org.netbeans.spi.lsp.CompletionCollector;
import org.enso.interpreter.runtime.Module;
import org.enso.interpreter.runtime.EnsoContext;
import org.enso.interpreter.runtime.callable.function.Function;
import org.enso.interpreter.runtime.data.Type;
import scala.Option;

@MimeRegistration(mimeType = "application/x-enso", service = CompletionCollector.class)
public class EnsoCompletionCollector implements CompletionCollector {
  @Override
  public boolean collectCompletions(Document document, int i, Completion.Context completionCtx, Consumer<Completion> consumer) {
    var ctx = EnsoContext.get(null);
    consumer.accept(CompletionCollector.newBuilder("Hi from Enso!").build());
    return true;
  }

  public List<String> getCompletions(String partialExpression) {
    CompletionContext completionCtx = extractCompletionContext(partialExpression);
    if (completionCtx.type != null) {
      Set<Function> methods =
          searchMethodsForType(completionCtx.type, completionCtx.prefixMethodOnType);
      return methods.stream()
          .map(Function::getName)
          .collect(Collectors.toList());
    } else {
      Set<Type> types = searchTypes(completionCtx.prefixMethodOnType);
      return types.stream()
          .map(Type::getName)
          .collect(Collectors.toList());
    }
  }
  private CompletionContext extractCompletionContext(String partialExpression) {
    var ctx = EnsoContext.get(null);
    assert ctx != null;
    var source = Source.newBuilder("enso", partialExpression, "<repl-completions>")
        .internal(true)
        .build();
    Tree parsedExpr = ctx.getCompiler().parseInline(source);
    if (parsedExpr instanceof Tree.BodyBlock bodyBlock) {
      if (bodyBlock.getStatements().size() != 1) {
        throw new IllegalStateException("Unexpected number of statements (lines) in the block: " + bodyBlock.getStatements().size());
      }
      var expr = bodyBlock.getStatements().get(0).getExpression();
      switch (expr) {
        case Tree.OprApp opr -> {
          if (opr.getOpr().getRight().codeRepr().equals(".")) {
            if (opr.getRhs() instanceof Ident ident) {
              String lhsExprSrc = opr.getLhs().codeRepr();
              //Object res = evalSrc(lhsExprSrc);
              Object res = null;
              var typesLib = TypesLibrary.getUncached();
              if (res != null && typesLib.hasType(res)) {
                return new CompletionContext(typesLib.getType(res), ident.getToken().codeRepr());
              } else {
                return new CompletionContext(null, ident.getToken().codeRepr());
              }
            } else {
              throw new IllegalStateException(
                  "Unexpected expression type: " + opr.getRhs().getClass());
            }
          }
        }
        case Tree.Ident ident ->  {
          return new CompletionContext(null, ident.getToken().codeRepr());
        }
        case default -> throw new IllegalStateException("Unexpected expression type: " + expr.getClass());
      }
    } else {
      throw new IllegalStateException("Unexpected expression type: " + parsedExpr.getClass());
    }
    throw new IllegalStateException("unreachable");
  }

  private Set<Type> searchTypes(String partialTypeName) {
    var ctx = EnsoContext.get(null);
    assert ctx != null;
    Collection<Module> allModules = ctx.getTopScope().getModules();
    Set<Type> typeCandidates = new HashSet<>();
    for (Module module : allModules) {
      Map<Type, Map<String, Function>> allMethods = module.getScope().getMethods();
      for (var entry : allMethods.entrySet()) {
        Type type = entry.getKey();
        if (type != null
            && type.getName() != null
            && type.getName().startsWith(partialTypeName)
            && !type.isEigenType()) {
          typeCandidates.add(type);
        }
      }
    }
    return typeCandidates;
  }

  private Set<Function> searchMethodsForType(Type type, String partialMethodName) {
    var ctx = EnsoContext.get(null);
    Collection<Module> allModules = ctx.getTopScope().getModules();
    Set<Function> methodCandidates = new HashSet<>();
    for (Module module : allModules) {
      Map<String, Function> typeMethods = module.getScope().getMethods().get(type);
      if (typeMethods != null) {
        for (Map.Entry<String, Function> methodEntry : typeMethods.entrySet()) {
          String methodName = methodEntry.getKey();
          Function method = methodEntry.getValue();
          if (methodName.startsWith(partialMethodName) && method != null) {
            methodCandidates.add(method);
          }
        }
      }
    }
    return methodCandidates;
  }

  private record CompletionContext(
      // May be null
      Type type,
      String prefixMethodOnType
  ) {}

  /*private Object evalSrc(String src) {
    var ctx = EnsoContext.get(null);

    LocalScope newLocalScope = nodeState.getLastScope().getLocalScope().createChild();
    ModuleScope moduleScope = nodeState.getLastScope().getModuleScope();
    var inlineContext = InlineContext.fromJava(
        newLocalScope,
        moduleScope,
        TailStatus.NOT_TAIL,
        ctx.getCompilerConfig()
    );
    Option<ExpressionNode> exprNode = ctx.getCompiler().runInline(src, inlineContext);
    if (exprNode.isEmpty() || exprNode.get() instanceof ErrorNode) {
      return null;
    } else {
      return exprNode.get().executeGeneric(nodeState.getLastScope().getFrame());
    }
  }*/
}
