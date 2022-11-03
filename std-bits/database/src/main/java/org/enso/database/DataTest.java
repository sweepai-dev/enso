package org.enso.database;

import java.io.BufferedReader;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Base64;
import java.io.File;
import java.util.List;

public class DataTest {
  private final static String CREDENTIALS_FILE = "C:\\Repos\\Enso\\TestData\\credentials.txt";

  public static String encode(String input) {
    return Base64.getEncoder().encodeToString(input.getBytes(StandardCharsets.UTF_8));
  }

  public static String decode(String input) {
    return new String(Base64.getDecoder().decode(input), StandardCharsets.UTF_8);
  }

  public static String[] getCredential(String name) {
    String output = null;

    var file = new File(CREDENTIALS_FILE);
    if (file.exists()) {
      try {
        var reader = new BufferedReader(new java.io.FileReader(file));
        String line = reader.readLine();
        while (line != null && !line.startsWith(name + ":")) {
          line = reader.readLine();
        }
        reader.close();

        if (line != null) {
          output = line.substring(name.length() + 1);
        }
      } catch (IOException ignored) {
      }
    }

    return output == null
        ? new String[0]
        : Arrays.stream(output.split(":")).map(DataTest::decode).toArray(String[]::new);
  }

  public static String[] listCredentials() {
    List<String> output = new ArrayList<>();

    var file = new File(CREDENTIALS_FILE);
    if (file.exists()) {
      try {
        var reader = new BufferedReader(new java.io.FileReader(file));
        String line = reader.readLine();
        while (line != null) {
          output.add(line.substring(0, line.indexOf(':')));
          line = reader.readLine();
        }
        reader.close();
      } catch (IOException ignored) {
      }
    }

    return output.toArray(String[]::new);
  }

  public static void setCredential(String name, String[] values) {
    String encoded = Arrays.stream(values).map(DataTest::encode).reduce((a, b) -> a + ":" + b).orElse("");

    List<String> output = new ArrayList<>();
    var file = new File(CREDENTIALS_FILE);
    if (file.exists()) {
      try {
        var reader = new BufferedReader(new java.io.FileReader(file));
        String line = reader.readLine();
        while (line != null) {
          if (!line.startsWith(name + ":")) {
            output.add(line);
          }
          line = reader.readLine();
        }
        reader.close();

        output.add(name + ":" + encoded);
      } catch (IOException ignored) {
      }
    }

    try {
      var writer = new java.io.FileWriter(file);
      for (String line : output) {
        writer.write(line + "\n");
      }
      writer.close();
    } catch (IOException ignored) {
    }
  }
}
