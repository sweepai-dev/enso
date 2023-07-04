package org.enso.interpreter.runtime.number;

import com.oracle.truffle.api.CompilerDirectives;
import com.oracle.truffle.api.interop.InteropLibrary;
import com.oracle.truffle.api.interop.TruffleObject;
import com.oracle.truffle.api.library.CachedLibrary;
import com.oracle.truffle.api.library.ExportLibrary;
import com.oracle.truffle.api.library.ExportMessage;
import org.enso.interpreter.runtime.EnsoContext;
import org.enso.interpreter.runtime.data.Type;
import org.enso.interpreter.runtime.library.dispatch.TypesLibrary;

import java.math.BigInteger;

/** Internal wrapper for a {@link BigInteger}. */
@ExportLibrary(InteropLibrary.class)
@ExportLibrary(TypesLibrary.class)
public final class EnsoBigInteger extends Number implements TruffleObject {
  private final BigInteger value;

  /**
   * Wraps a {@link BigInteger}.
   *
   * @param value the value to wrap.
   */
  public EnsoBigInteger(BigInteger value) {
    assert (value.bitLength() > 63);
    this.value = value;
  }

  /** @return the contained {@link BigInteger}. */
  public BigInteger getValue() {
    return value;
  }

  @Override
  @CompilerDirectives.TruffleBoundary
  public String toString() {
    return value.toString();
  }

  @CompilerDirectives.TruffleBoundary
  @ExportMessage
  String toDisplayString(boolean allowSideEffects) {
    return value.toString();
  }

  @ExportMessage
  boolean isNumber() {
    return true;
  }

  @ExportMessage
  boolean fitsInBigInteger() {
    return true;
  }

  @ExportMessage
  @CompilerDirectives.TruffleBoundary
  byte asByte() {
    return byteValue();
  }

  @ExportMessage
  @CompilerDirectives.TruffleBoundary
  short asShort() {
    return shortValue();
  }

  @ExportMessage
  @CompilerDirectives.TruffleBoundary
  int asInt() {
    return intValue();
  }

  @ExportMessage
  @CompilerDirectives.TruffleBoundary
  long asLong() {
    return longValue();
  }

  @ExportMessage
  @CompilerDirectives.TruffleBoundary
  float asFloat() {
    return floatValue();
  }

  @ExportMessage
  @CompilerDirectives.TruffleBoundary
  double asDouble() {
    return doubleValue();
  }

  @ExportMessage
  BigInteger asBigInteger() {
    return value;
  }

  @ExportMessage
  Type getMetaObject(@CachedLibrary("this") InteropLibrary thisLib) {
    return EnsoContext.get(thisLib).getBuiltins().number().getBigInteger();
  }

  @ExportMessage
  boolean hasMetaObject() {
    return true;
  }

  @ExportMessage
  boolean hasType() {
    return true;
  }

  @ExportMessage
  Type getType(@CachedLibrary("this") TypesLibrary thisLib) {
    return EnsoContext.get(thisLib).getBuiltins().number().getBigInteger();
  }

  @Override
  public int intValue() {
    return value.intValue();
  }

  @Override
  public long longValue() {
    return value.longValue();
  }

  @Override
  public float floatValue() {
    return value.floatValue();
  }

  @Override
  public double doubleValue() {
    return value.doubleValue();
  }

  @Override
  public boolean equals(Object obj) {
    if (obj instanceof EnsoBigInteger otherBigInt) {
      return value.equals(otherBigInt.value);
    } else {
      return false;
    }
  }
}
