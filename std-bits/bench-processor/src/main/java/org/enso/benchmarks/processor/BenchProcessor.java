package org.enso.benchmarks.processor;

import java.io.File;
import java.io.IOException;
import java.io.Writer;
import java.net.URISyntaxException;
import java.util.Collection;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;
import javax.annotation.processing.AbstractProcessor;
import javax.annotation.processing.Processor;
import javax.annotation.processing.RoundEnvironment;
import javax.annotation.processing.SupportedAnnotationTypes;
import javax.annotation.processing.SupportedSourceVersion;
import javax.lang.model.SourceVersion;
import javax.lang.model.element.TypeElement;
import org.enso.benchmarks.BenchGroup;
import org.enso.benchmarks.BenchSpec;
import org.enso.benchmarks.BenchSuiteWrapper;
import org.graalvm.polyglot.Engine;
import org.openide.util.lookup.ServiceProvider;

@SupportedAnnotationTypes("org.enso.benchmarks.processor.GenerateBenchSources")
@SupportedSourceVersion(SourceVersion.RELEASE_17)
@ServiceProvider(service = Processor.class)
public class BenchProcessor extends AbstractProcessor {

  private final File ensoHomeOverride;
  private File ensoDir;
  private final File benchRootDir;
  private final SpecCollector specCollector;
  private static final String generatedSourcesPackagePrefix = "org.enso.benchmarks.libs.generated";
  private static final List<String> imports = List.of(
      "import java.nio.file.Paths;",
      "import java.io.ByteArrayOutputStream;",
      "import java.io.File;",
      "import org.openjdk.jmh.annotations.Benchmark;",
      "import org.openjdk.jmh.annotations.BenchmarkMode;",
      "import org.openjdk.jmh.annotations.Mode;",
      "import org.openjdk.jmh.annotations.Fork;",
      "import org.openjdk.jmh.annotations.Measurement;",
      "import org.openjdk.jmh.annotations.OutputTimeUnit;",
      "import org.openjdk.jmh.annotations.Setup;",
      "import org.openjdk.jmh.annotations.State;",
      "import org.openjdk.jmh.annotations.Scope;",
      "import org.openjdk.jmh.infra.BenchmarkParams;",
      "import org.openjdk.jmh.infra.Blackhole;",
      "import org.graalvm.polyglot.Context;",
      "import org.graalvm.polyglot.Value;",
      "import org.graalvm.polyglot.io.IOAccess;",
      "import org.enso.polyglot.LanguageInfo;",
      "import org.enso.polyglot.MethodNames;",
      "import org.enso.benchmarks.processor.SpecCollector;",
      "import org.enso.benchmarks.BenchSuiteWrapper;",
      "import org.enso.benchmarks.BenchSpec;",
      "import org.enso.benchmarks.BenchGroup;"
  );

  public BenchProcessor() {
    System.out.println("Initializing BenchProcessor");
    var langs = Engine.create().getLanguages();
    System.out.println("Languages: ");
    for (var lang : langs.keySet()) {
      System.out.println("  " + lang);
    }
    try {
      ensoDir = new File(
          BenchProcessor.class
              .getProtectionDomain()
              .getCodeSource()
              .getLocation()
              .toURI()
      );
    } catch (URISyntaxException e) {
      throw new IllegalStateException("Unreachable", e);
    }
    for (; ensoDir != null; ensoDir = ensoDir.getParentFile()) {
      if (ensoDir.getName().equals("enso")) {
        break;
      }
    }
    if (ensoDir == null) {
      throw new IllegalStateException("Unreachable: Could not find Enso root directory");
    }

    benchRootDir = ensoDir.toPath()
        .resolve("test")
        .resolve("Benchmarks")
        .toFile();
    if (!benchRootDir.isDirectory() || !benchRootDir.canRead()) {
      throw new IllegalStateException("Unreachable: Could not find Enso benchmarks directory");
    }

    // Note that ensoHomeOverride does not have to exist, only its parent directory
    ensoHomeOverride = ensoDir.toPath()
        .resolve("distribution")
        .resolve("component")
        .toFile();
    specCollector =
        new SpecCollector(benchRootDir, ensoHomeOverride);
  }

  @Override
  public boolean process(Set<? extends TypeElement> annotations, RoundEnvironment roundEnv) {
    System.out.println("[mylog] Running BenchProcessor");
    System.out.println("[mylog] ensoRootDir: " + ensoDir);
    Collection<BenchSuiteWrapper> benchSuites = specCollector.collectAllBenchSpecs();
    for (BenchSuiteWrapper benchSuite : benchSuites) {
      for (BenchGroup group : benchSuite.getGroups()) {
        generateClassForGroup(group, benchSuite.getModuleQualifiedName());
      }
    }
    return true;
  }

  private void generateClassForGroup(BenchGroup group, String moduleName) {
    String fullClassName = createGroupClassName(group);
    try (Writer srcFileWriter = processingEnv.getFiler().createSourceFile(fullClassName).openWriter()) {
      generateClassForGroup(srcFileWriter, moduleName, group);
    } catch (IOException e) {
      System.err.println("Failed to generate source file for group '" + group.name() + "': " + e.getMessage());
    }
  }

  private void generateClassForGroup(Writer javaSrcFileWriter, String moduleName, BenchGroup group) throws IOException {
    String groupFullClassName = createGroupClassName(group);
    System.out.println("[mylog] Generating spec code for group '" + groupFullClassName + "'");
    String className = groupFullClassName.substring(groupFullClassName.lastIndexOf('.') + 1);
    List<BenchSpec> specs = group.specs();
    List<String> specJavaNames = specs
        .stream()
        .map(spec -> normalize(spec.name()))
        .collect(Collectors.toUnmodifiableList());

    javaSrcFileWriter.append("package " + generatedSourcesPackagePrefix + ";\n");
    javaSrcFileWriter.append("\n");
    javaSrcFileWriter.append(String.join("\n", imports));
    javaSrcFileWriter.append("\n");
    javaSrcFileWriter.append("\n");
    javaSrcFileWriter.append("/**\n");
    javaSrcFileWriter.append(" * Generated from:\n");
    javaSrcFileWriter.append(" * - Module: " + moduleName + "\n");
    javaSrcFileWriter.append(" * - Group: \"" + group.name() + "\"\n");
    javaSrcFileWriter.append(" * Generated by {@link " + getClass().getName() + "}.\n");
    javaSrcFileWriter.append(" */\n");
    javaSrcFileWriter.append("@BenchmarkMode(Mode.AverageTime)\n");
    javaSrcFileWriter.append("@Fork(1)\n");
    javaSrcFileWriter.append("@State(Scope.Benchmark)\n");
    javaSrcFileWriter.append("public class " + className + " {\n");
    javaSrcFileWriter.append("  private Value groupInputArg;\n");
    for (var specJavaName : specJavaNames) {
      javaSrcFileWriter.append("  private Value benchFunc_" + specJavaName + ";\n");
    }
    javaSrcFileWriter.append("  \n");
    javaSrcFileWriter.append("  @Setup\n");
    javaSrcFileWriter.append("  public void setup(BenchmarkParams params) throws Exception {\n");
    javaSrcFileWriter.append("    var ctx = Context.newBuilder()\n");
    javaSrcFileWriter.append("      .allowExperimentalOptions(true)\n");
    javaSrcFileWriter.append("      .allowIO(IOAccess.ALL)\n");
    javaSrcFileWriter.append("      .allowAllAccess(true)\n");
    javaSrcFileWriter.append("      .logHandler(new ByteArrayOutputStream())\n");
    javaSrcFileWriter.append("      .option(\n");
    javaSrcFileWriter.append("        \"enso.languageHomeOverride\",\n");
    javaSrcFileWriter.append("        Paths.get(\"../../distribution/component\").toFile().getAbsolutePath()\n");
    javaSrcFileWriter.append("      ).build();\n");
    javaSrcFileWriter.append("    \n");
    javaSrcFileWriter.append("    Value module = ctx.getBindings(LanguageInfo.ID).invokeMember(MethodNames.TopScope.GET_MODULE, \"" + moduleName + "\");\n");
    javaSrcFileWriter.append("    File benchProjectDir = new File(\"" + benchRootDir.getAbsolutePath() + "\");\n");
    javaSrcFileWriter.append("    File languageHomeOverride = new File(\"" + ensoHomeOverride.getAbsolutePath() + "\");\n");
    javaSrcFileWriter.append("    var specCollector = new SpecCollector(benchProjectDir, languageHomeOverride);\n");
    javaSrcFileWriter.append("    BenchSuiteWrapper benchSuite = specCollector.collectBenchSpecFromModuleName(\"" + moduleName + "\");\n");
    javaSrcFileWriter.append("    \n");
    for (int i = 0; i < specs.size(); i++) {
      var specJavaName = specJavaNames.get(i);
      var specName = specs.get(i).name();
      javaSrcFileWriter.append("    BenchSpec benchSpec_" + specJavaName + " = benchSuite.findSpecByName(\"" + group.name() + "\", \"" + specName + "\");\n");
      javaSrcFileWriter.append("    this.benchFunc_" + specJavaName + " = benchSpec_" + specJavaName + ".code();\n");
    }
    javaSrcFileWriter.append("    \n");
    javaSrcFileWriter.append("    this.groupInputArg = benchSuite.getDefaultInputArgument();\n");
    javaSrcFileWriter.append("  } \n"); // end of setup method
    javaSrcFileWriter.append("  \n");
    for (var specJavaName : specJavaNames) {
      javaSrcFileWriter.append("  \n");
      javaSrcFileWriter.append("  @Benchmark\n");
      javaSrcFileWriter.append("  public void " + specJavaName + "(Blackhole blackhole) {\n");
      javaSrcFileWriter.append("    Value result = this.benchFunc_" + specJavaName + ".execute(this.groupInputArg);\n");
      javaSrcFileWriter.append("    blackhole.consume(result);\n");
      javaSrcFileWriter.append("  }\n"); // end of benchmark method
    }
    javaSrcFileWriter.append("}\n"); // end of class className
  }

  /**
   * Returns Java FQN for a benchmark spec.
   * @param group Group name will be converted to Java package name.
   * @return
   */
  private static String createGroupClassName(BenchGroup group) {
    var groupPkgName = normalize(group.name());
    return generatedSourcesPackagePrefix + "." + groupPkgName;
  }

  private static boolean isValidChar(char c) {
    return Character.isAlphabetic(c) || Character.isDigit(c) || c == '_';
  }

  /**
   * Converts Text to valid Java identifier.
   * @param name Text to convert.
   * @return Valid Java identifier, non null.
   */
  private static String normalize(String name) {
    var normalizedNameSb = new StringBuilder();
    for (char c : name.toCharArray()) {
      if (isValidChar(c)) {
        normalizedNameSb.append(c);
      } else if (Character.isWhitespace(c) &&  (peekLastChar(normalizedNameSb) != '_')) {
        normalizedNameSb.append('_');
      }
    }
    return normalizedNameSb.toString();
  }

  private static char peekLastChar(StringBuilder sb) {
    if (sb.length() > 0) {
      return sb.charAt(sb.length() - 1);
    } else {
      return 0;
    }
  }
}
