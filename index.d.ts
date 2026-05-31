// @ts-nocheck
export * from './js-bindings';

/**
 * Determines if the given chunk data indicates the end of the SVG output.
 * Useful for callback-based vectorization to know when the final chunk has been received.
 */
export function isEOF(chunk: string, progress: number): boolean;
