package org.enso.table.parsing;

import org.enso.table.data.column.builder.Builder;
import org.enso.table.data.column.builder.NumericBuilder;
import org.enso.table.data.column.storage.Storage;
import org.enso.table.parsing.problems.ProblemAggregator;
import org.enso.table.parsing.problems.ProblemAggregatorImpl;
import org.enso.table.problems.WithProblems;
import org.graalvm.polyglot.Context;

import java.util.Collections;
import java.util.HashMap;
import java.util.Map;
import java.util.regex.Pattern;

/** A parser for numbers.
 *
 * This parser will attempt to work out what the decimal point and thousand
 * separators used in the input. It will try various ways of formatting a number
 * and can be set to allow for scientific notation, currency symbols.
 *
 * If parsing a column it will select the format that parses the longest set
 * without an issue from the top and then apply this format to all the rows.
 *
 * The separators will be tried in British, German, French and Swiss order.
 * - Thousand separator must be followed by groups of 3 numbers.
 * - Scientific notation is only allowed on decimals and must be on a value
 *   between -10 and 10. The notation is an `E` followed by an integer.
 *
 * The following formats are supported:
 * - Sign (+/-) followed by Number (e.g. +1,234.56)
 * - Using brackets to indicate a negative number (e.g. (1,234.56))
 * - Currency symbols (if enabled) can be placed before or after the sign and
 *   number.
 * - If using brackets, the currency symbol must be placed after the opening
 *   bracket.
 * */
public class NumberParser extends IncrementalDatatypeParser {
    private final static String SIGN = "(?<sign>[-+])?";
    private final static String BRACKETS = "(?<sign>\\((?=.*\\)\\s*$))?\\s*";
    private final static String BRACKET_CLOSE = "\\)?";
    private final static String CCY = "(?<ccy>[^0-9(),. '+-]+)";
    private final static String EXP = "(?<exp>[eE][+-]?\\d+)?";
    private final static String SPACE = "\\s*";
    private final static String[] SEPARATORS = new String[] {",.", ".,", " ,", "',"};

    private final static Map<String, Pattern> PATTERNS = new HashMap<>();

    private static Pattern getPattern(boolean allowDecimal, int allowedPatterns, boolean allowScientific, boolean trimValues, int index) {
        if (allowedPatterns == 0) {
            return  null;
        }

        int separatorsIndex = index / allowedPatterns;
        int patternIndex = index % allowedPatterns;

        if (separatorsIndex >= SEPARATORS.length) {
            return null;
        }

        var separators = SEPARATORS[separatorsIndex];
        return getPattern(allowDecimal, allowedPatterns, allowScientific, trimValues, patternIndex, separators);
    }

    /** The number of patterns that are allowed for non-currency numbers. */
    private static final int ALLOWED_NON_CCY_PATTERNS = 2;

    /** The number of patterns that are allowed for currency numbers. */
    private static final int ALLOWED_CCY_PATTERNS = 6;

    private static Pattern getPattern(boolean allowDecimal, int allowedPatterns, boolean allowScientific, boolean trimValues, int patternIndex, String separators) {
        if (allowScientific && !allowDecimal) {
            throw new IllegalArgumentException("Scientific notation requires decimal numbers.");
        }

        if (patternIndex >= allowedPatterns) {
            return null;
        }

        var INTEGER = "(?<integer>(\\d*)" + (separators.length() == 1 ? "" : "|(\\d{1,3}([" + separators.charAt(0) + "]\\d{3})*)") + ")";

        var decimalPoint = (separators.length() == 1 ? separators : separators.charAt(1));
        var NUMBER = INTEGER + (allowDecimal ? "(?<decimal>[" + decimalPoint + "]\\d*)?" : "") + (allowScientific ? EXP : "");

        var pattern = switch (patternIndex) {
            case 0 -> SIGN + NUMBER;
            case 1 -> BRACKETS + NUMBER + BRACKET_CLOSE;
            case 2 -> SIGN + CCY + SPACE + NUMBER;
            case 3 -> CCY + SPACE + SIGN + NUMBER;
            case 4 -> SIGN + NUMBER + CCY;
            case 5 -> BRACKETS + CCY + SPACE + NUMBER + BRACKET_CLOSE;
            default -> throw new IllegalArgumentException("Invalid pattern index: " + patternIndex);
        };

        if (trimValues) {
            pattern = SPACE + pattern + SPACE;
        }

        return PATTERNS.computeIfAbsent("^" + pattern + "$", Pattern::compile);
    }

    private final boolean allowDecimal;
    private final int allowedPatterns;
    private final boolean allowLeadingZeros;
    private final boolean allowScientific;
    private final boolean trimValues;
    private final String separators;

    /**
     * Creates a new integer instance of this parser.
     *
     * @param allowCurrency whether to allow currency symbols
     * @param allowLeadingZeros whether to allow leading zeros
     * @param trimValues whether to trim the input values
     * @param thousandSeparator the thousand separator to use
     */
    public static NumberParser createIntegerParser(boolean allowCurrency, boolean allowLeadingZeros, boolean trimValues, String thousandSeparator) {
        var separator = thousandSeparator == null ? null : (thousandSeparator + '_');
        var allowedPatterns = allowCurrency ? ALLOWED_CCY_PATTERNS : ALLOWED_NON_CCY_PATTERNS;
        return new NumberParser(false, allowedPatterns, allowLeadingZeros, trimValues, false, separator);
    }

    public  static  NumberParser createFastOnlyParser(boolean allowDecimal, boolean trimValues) {
        return new NumberParser(allowDecimal, 0, false, trimValues, false, null);
    }

    /**
     * Creates a new decimal instance of this parser.
     *
     * @param allowCurrency whether to allow currency symbols
     * @param allowLeadingZeros whether to allow leading zeros
     * @param trimValues whether to trim the input values
     * @param allowScientific whether to allow scientific notation
     */
    public static NumberParser createAutoDecimalParser(boolean allowCurrency, boolean allowLeadingZeros, boolean trimValues, boolean allowScientific) {
        var allowedPatterns = allowCurrency ? ALLOWED_CCY_PATTERNS : ALLOWED_NON_CCY_PATTERNS;
        return new NumberParser(true, allowedPatterns, allowLeadingZeros, trimValues, allowScientific, null);
    }

    /**
     * Creates a new decimal instance of this parser.
     *
     * @param allowCurrency whether to allow currency symbols
     * @param allowLeadingZeros whether to allow leading zeros
     * @param trimValues whether to trim the input values
     * @param allowScientific whether to allow scientific notation
     * @param thousandSeparator the thousand separator to use
     * @param decimalSeparator the decimal separator to use
     */
    public static NumberParser createFixedDecimalParser(boolean allowCurrency, boolean allowLeadingZeros, boolean trimValues, boolean allowScientific, String thousandSeparator, String decimalSeparator) {
        if (decimalSeparator == null || decimalSeparator.length() != 1) {
            throw new IllegalArgumentException("Decimal separator must be a single character.");
        }

        thousandSeparator = thousandSeparator == null ? "" : thousandSeparator;

        var allowedPatterns = allowCurrency ? ALLOWED_CCY_PATTERNS : ALLOWED_NON_CCY_PATTERNS;
        return new NumberParser(true, allowedPatterns, allowLeadingZeros, trimValues, allowScientific, thousandSeparator + decimalSeparator);
    }

    private NumberParser(boolean allowDecimal, int allowedPatterns, boolean allowLeadingZeros, boolean trimValues, boolean allowScientific, String separators) {
        this.allowDecimal = allowDecimal;
        this.allowedPatterns = allowedPatterns;
        this.allowLeadingZeros = allowLeadingZeros;
        this.trimValues = trimValues;
        this.allowScientific = allowScientific;
        this.separators = separators;
    }

    /**
     * Creates a Pattern for the given index.
     * The index will be decoded into a specific set of separators (unless fixed
     * separators are used) and then paired with on of the valid patterns for
     * the given parser.
     */
    private Pattern patternForIndex(int index) {
        if (index == 0) {
            return PATTERNS.computeIfAbsent("^.*$", Pattern::compile);
        }

        return separators == null
            ? getPattern(allowDecimal, allowedPatterns, allowScientific, trimValues, index-1)
            : getPattern(allowDecimal, allowedPatterns, allowScientific, trimValues, index-1, separators);
    }

    @Override
    protected Object parseSingleValue(String text, ProblemAggregator problemAggregator) {
        int index = 0;
        var pattern = patternForIndex(index);
        while (pattern != null) {
            var value = innerParseSingleValue(text, pattern, index);
            if (value != null) {
                return value;
            }

            index++;
            pattern = patternForIndex(index);
        }

        problemAggregator.reportInvalidFormat(text);
        return null;
    }

    @Override
    public WithProblems<Storage<?>> parseColumn(String columnName, Storage<String> sourceStorage) {
        int index = 0;
        var pattern = patternForIndex(index);

        int bestIndex = 0;
        int bestCount = -1;
        while (pattern != null) {
            Builder builder = makeBuilderWithCapacity(sourceStorage.size());
            int failedAt = parseColumnWithPattern(pattern, index, sourceStorage, builder, null);
            if (failedAt == -1) {
                return new WithProblems<>(builder.seal(), Collections.emptyList());
            }

            if (failedAt > bestCount) {
                bestCount = failedAt;
                bestIndex = index;
            }

            index++;
            pattern = patternForIndex(index);
        }

        Builder fallback = makeBuilderWithCapacity(sourceStorage.size());
        ProblemAggregator aggregator = new ProblemAggregatorImpl(columnName);
        parseColumnWithPattern(patternForIndex(bestIndex), bestIndex, sourceStorage, fallback, aggregator);
        return new WithProblems<>(fallback.seal(), aggregator.getAggregatedProblems());
    }

    private int parseColumnWithPattern(Pattern pattern, int index, Storage<String> sourceStorage, Builder builder, ProblemAggregator aggregator) {
        Context context = Context.getCurrent();
        for (int i = 0; i < sourceStorage.size(); i++) {
            var text = sourceStorage.getItemBoxed(i);
            if (text == null) {
                builder.appendNulls(1);
            } else {
                var value = innerParseSingleValue(text, pattern, index);
                if (value != null) {
                    builder.appendNoGrow(value);
                } else {
                    if (aggregator == null) {
                        return i;
                    }

                    aggregator.reportInvalidFormat(text);
                    builder.appendNulls(1);
                }
            }

            context.safepoint();
        }
        return -1;
    }

    @Override
    protected Builder makeBuilderWithCapacity(int capacity) {
        return allowDecimal
                ? NumericBuilder.createDoubleBuilder(capacity)
                : NumericBuilder.createLongBuilder(capacity);
    }

    private Object innerParseSingleValue(String text, Pattern pattern, int index) {
        if (allowDecimal) {
            var trimmed = trimValues ? text.trim() : text;
            if (trimmed.equals("NaN")) {
                return Double.NaN;
            }
            if (trimmed.equals("Infinity")) {
                return Double.POSITIVE_INFINITY;
            }
            if (trimmed.equals("-Infinity")) {
                return Double.NEGATIVE_INFINITY;
            }

            if (index == 0) {
                try {
                    return Double.parseDouble(trimmed);
                } catch (NumberFormatException e) {
                    return null;
                }
            }
        } else if (index == 0) {
            return fastLongValue(text);
        }

        var parsed = pattern.matcher(text);
        if (!parsed.matches()) {
            return null;
        }

        try {
            var sign = parsed.group("sign");
            var sign_value = sign != null && !sign.equals("+") ? -1 : 1;

            var integer = parsed.group("integer").replaceAll("\\D", "");

            if (!allowLeadingZeros && integer.length() > 1 && integer.charAt(0) == '0') {
                return null;
            }

            if (allowDecimal) {
                var decimal = parsed.group("decimal");
                var decimalPrepared = decimal == null ? "" : ("." + decimal.substring(1));

                if (integer.equals("") && decimalPrepared.equals("")) {
                    return null;
                }

                integer = integer.equals("") ? "0" : integer;

                if (allowScientific) {
                    var exp = parsed.group("exp");
                    if (exp != null) {
                        if (integer.length() > 1) {
                            return null;
                        }
                        decimalPrepared = decimalPrepared + exp;
                    }
                }

                return sign_value * Double.parseDouble(integer + decimalPrepared);
            }

            return integer.equals("") ? null : sign_value * Long.parseLong(integer);
        } catch (NumberFormatException e) {
            throw new IllegalStateException("Java parse failed to parse number: " + text, e);
        }
    }

    private static int SkipWhiteSpace(String text, int idx, int length) {
        while (idx < length && Character.isWhitespace(text.charAt(idx))) {
            idx++;
        }
        return idx;
    }

    private static long MAX_ALLOWED_LONG = Long.MAX_VALUE / 10;

    /**
     * Parses a long value from the given text.
     * Very simple and direct and avoids throwing exceptions.
     *
     * @param text the text to parse
     * @return the parsed value or null if the text is not a valid long
     */
    private Long fastLongValue(String text) {
        if (text == null || text.isEmpty()) {
            return null;
        }

        int length = text.length();
        int idx = this.trimValues ? SkipWhiteSpace(text, 0, length) : 0;
        if (idx == length) {
            return null;
        }

        boolean negative = text.charAt(idx) == '-';
        if (negative) {
            idx++;
        }
        if (idx == length) {
            return null;
        }

        long accum = 0;
        while (idx < length && Character.isWhitespace(text.charAt(idx))) {
            char c = text.charAt(idx++);
            if (c < '0' || c > '9') {
                return null;
            }

            int digit = c - '0';
            if (accum > MAX_ALLOWED_LONG) {
                return null;
            }
            accum *= 10;
            if (accum > Long.MAX_VALUE - digit) {
                return null;
            }
            accum += digit;
        }

        idx = this.trimValues ? SkipWhiteSpace(text, idx, length) : idx;
        return idx == length ? (negative ? -accum : accum) : null;
    }
}
