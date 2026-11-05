/** Shared numeric wire constraints for build-time and generated browser validators. */
/** @type {Record<string, import('ajv').FormatDefinition<number>>} */
export const numericFormats = {
  uint32: { type: 'number', validate: value => Number.isInteger(value) && value >= 0 && value <= 0xffff_ffff },
  uint64: { type: 'number', validate: value => Number.isSafeInteger(value) && value >= 0 },
};
