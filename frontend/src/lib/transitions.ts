import { cubicOut } from "svelte/easing";
import type { EasingFunction, TransitionConfig } from "svelte/transition";

export const ROLL_MS = 400;

type RollParams = {
  delay?: number;
  duration?: number;
  easing?: EasingFunction;
  distance?: number;
};

/**
 * Slides a row's contents vertically while its wrapper's height animates
 *
 * sign is the side the contents sit on at the start of an intro (and the side
 * they leave towards during an outro): 1 is below the row, -1 is above it.
 */
const roll =
  (sign: 1 | -1) =>
  (
    node: HTMLElement,
    { delay = 0, duration = ROLL_MS, easing = cubicOut, distance = 0.5 }: RollParams = {},
  ): TransitionConfig => {
    const travel = node.offsetHeight * distance;
    return {
      delay,
      duration,
      easing,
      css: (t, u) => `transform: translateY(${sign * u * travel}px); opacity: ${t};`,
    };
  };

export const rollIn = roll(1);
export const rollOut = roll(-1);
