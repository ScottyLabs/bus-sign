/** the threshold where arrival ETA shows "NOW" instead of a number */
const IMMINENT_SECONDS = 60;

export const formatMinutes = (seconds: number): string =>
  seconds < IMMINENT_SECONDS ? "NOW" : `${Math.ceil(seconds / 60)} min`;

/** Arrivals that have the same minute should sit in the same 'bucket' for
 * ordering so that it doesn't seem like they randomly swap around
 */
export const arrivalBucket = (seconds: number): number =>
  seconds < IMMINENT_SECONDS ? 0 : Math.ceil(seconds / 60);
