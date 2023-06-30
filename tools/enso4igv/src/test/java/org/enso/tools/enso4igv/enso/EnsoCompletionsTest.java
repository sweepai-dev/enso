package org.enso.tools.enso4igv.enso;

import java.util.ArrayList;
import java.util.List;
import javax.swing.text.AbstractDocument.Content;
import javax.swing.text.BadLocationException;
import javax.swing.text.Document;
import javax.swing.text.PlainDocument;
import org.junit.Assert;
import org.junit.Test;
import org.netbeans.api.lsp.Completion;

public class EnsoCompletionsTest {
  @Test
  public void simpleCompletionTest() throws BadLocationException {
    Document doc = new PlainDocument();
    doc.insertString(0, """
        from Standard.Base import Vector
        main = Vector.len
        """, null);
    List<Completion> completions = new ArrayList<>();
    Completion.collect(doc, 49, null, completions::add);
    Assert.assertNotNull(completions);
  }

}
