/** the threshold where arrival ETA shows "NOW" instead of a number */
const IMMINENT_SECONDS = 60;

export const formatMinutes = (seconds: number): string =>
  seconds < IMMINENT_SECONDS ? "NOW" : `${Math.ceil(seconds / 60)} min`;

/** Arrivals that have the same minute should sit in the same 'bucket' for
 * ordering so that it doesn't seem like they randomly swap around
 */
export const arrivalBucket = (seconds: number): number =>
  seconds < IMMINENT_SECONDS ? 0 : Math.ceil(seconds / 60);

export type SortableArrivalEntry = {
  route: string;
  destination: string;
  arrivals: { seconds: number }[];
};

/** Previous board order for a stop, keyed by `route:destination`. */
export type ArrivalOrderTracker = Map<string, number>;

export const arrivalEntryKey = (entry: { route: string; destination: string }): string =>
  `${entry.route}:${entry.destination}`;

const nextBucket = (entry: SortableArrivalEntry): number => {
  const next = entry.arrivals[0];
  return next === undefined ? Number.MAX_SAFE_INTEGER : arrivalBucket(next.seconds);
};

/**
 * Sort by minute bucket, then by prior board position so a bus that just
 * entered an occupied minute stays below buses already showing that minute.
 */
export const sortByArrival = <T extends SortableArrivalEntry>(
  entries: T[],
  previousOrder: ArrivalOrderTracker,
): { sorted: T[]; order: ArrivalOrderTracker } => {
  const sorted = entries.toSorted((a, b) => {
    const bucketDiff = nextBucket(a) - nextBucket(b);
    if (bucketDiff !== 0) return bucketDiff;

    const rankA = previousOrder.get(arrivalEntryKey(a)) ?? Number.POSITIVE_INFINITY;
    const rankB = previousOrder.get(arrivalEntryKey(b)) ?? Number.POSITIVE_INFINITY;
    if (rankA !== rankB) return rankA - rankB;

    const routeDiff = a.route.localeCompare(b.route, "en", { numeric: true });
    if (routeDiff !== 0) return routeDiff;

    return a.destination.localeCompare(b.destination);
  });

  const order: ArrivalOrderTracker = new Map(
    sorted.map((entry, index) => [arrivalEntryKey(entry), index]),
  );

  return { sorted, order };
};
