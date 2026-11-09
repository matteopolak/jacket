import type { CaseResult, CatalogueCase } from './play_types.ts';
import { playgroundContract } from './playground_contract.ts';
import type { ContractResult } from './playground_contract.ts';
// Expected results use the mode-overlaid catalogue. Diagnostics cannot change eligibility.
export function playgroundCaseResult(
  test: Partial<CatalogueCase>,
  result: CaseResult,
): ContractResult {
  if (result.requestFailure !== undefined) {
    const request = result.request;
    const detail = request
      ? `; input files=${request.fileCount} bytes=${request.totalBytes} (paths=${request.pathBytes}, contents=${request.sourceBytes}, main=${request.mainBytes})`
      : '';
    return {
      pass: false,
      why:
        (result.compilerCrashed ? 'compiler crash: ' : 'request rejected: ') +
        result.requestFailure +
        detail,
    };
  }
  if (result.compilerCrashed) return { pass: false, why: 'compiler crash' };
  return playgroundContract(test, result);
}
