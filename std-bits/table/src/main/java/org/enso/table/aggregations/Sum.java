package org.enso.table.aggregations;

import org.enso.table.data.column.storage.DoubleStorage;
import org.enso.table.data.column.storage.LongStorage;
import org.enso.table.data.column.storage.Storage;
import org.enso.table.data.table.Column;
import org.enso.table.data.table.problems.InvalidAggregation;

import java.util.BitSet;
import java.util.List;

/***
 * Aggregate Column computing the total value in a group.
 */
public class Sum extends Aggregator {
  private final Storage storage;

  public Sum(String name, Column column) {
    super(name, Storage.Type.DOUBLE);
    this.storage = column.getStorage();
  }

  @Override
  public Object aggregate(List<Integer> indexes) {
    if (storage instanceof LongStorage) {
      return aggregateLong((LongStorage)storage, indexes);
    }

    if (storage instanceof DoubleStorage) {
      return aggregateDouble((DoubleStorage)storage, indexes);
    }

    return aggregateObject(indexes);
  }

  private Object aggregateObject(List<Integer> indexes) {
    Object current = null;
    for (int row : indexes) {
      Object value = storage.getItemBoxed(row);
      if (value != null) {
        if (current == null) {
          current = 0L;
        }

        Long lCurrent = CastToLong(current);
        Long lValue = CastToLong(value);
        if (lCurrent != null && lValue != null) {
          current = lCurrent + lValue;
        } else {
          Double dCurrent = CastToDouble(current);
          Double dValue = CastToDouble(value);
          if (dCurrent != null && dValue != null) {
            current = dCurrent + dValue;
          } else {
            this.addProblem(new InvalidAggregation(this.getName(), row, "Cannot convert to a number."));
            return null;
          }
        }
      }
    }
    return current;
  }

  private Long aggregateLong(LongStorage longStorage, List<Integer> indexes) {
    BitSet isMissing = longStorage.getIsMissing();

    boolean any = false;
    long total = 0L;

    for (int row : indexes) {
      if (!isMissing.get(row)) {
        total += longStorage.getItem(row);
        any = true;
      }
    }

    return any ? total : null;
  }

  private Double aggregateDouble(DoubleStorage doubleStorage, List<Integer> indexes) {
    BitSet isMissing = doubleStorage.getIsMissing();

    boolean any = false;
    double total = 0.0;

    for (int row : indexes) {
      if (!isMissing.get(row)) {
        total += doubleStorage.getItem(row);
        any = true;
      }
    }

    return any ? total : null;
  }
}
