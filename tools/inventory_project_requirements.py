#!/usr/bin/env python3
"""Static, pinned newer-project requirements beyond the small fixture matrix."""
from __future__ import annotations

import argparse
from bisect import bisect_right
from collections import Counter
import json
from pathlib import Path
import re

from check_corpus import ROOT, digest, inventory
from inventory_corpus_features import decode, mask_noncode

# Priorities are integration order, not claims that these constructs are unsupported.
REQUIREMENTS = {
    'module-parameters-and-target-selection': (0, 'module_parameters_high; parent target bridge', r'#module_parameters\b|#if\s+(?:OS|CPU)\b', ['module-instance-parameters', 'conditional-source']),
    'compiler-workspaces-and-message-lifecycle': (0, 'compiler_intrinsics_high; parent compiler effects', r'\bcompiler_(?:create_workspace|begin_intercept|wait_for_message|end_intercept|custom_link_command_is_complete)\b', []),
    'compiler-options-and-native-output': (0, 'parent compiler effects; llvm_types_high', r'\b(?:Build_Options|set_build_options|get_build_options|compiler_add_library_search_directory|compiler_generated_object_files)\b', []),
    'source-insertion-and-code-scope': (0, 'reflection_metaprogram_max; source_run_high', r'#(?:insert|code|expand)\b', ['source-code-capture']),
    'reflection-and-type-metadata': (0, 'reflection_metaprogram_max', r'\b(?:Type_Info(?:_\w+)?|type_info|type_of|initializer_of|members_of|make_location)\b', ['reflection-layout', 'reflection-initializer']),
    'polymorphic-and-baked-procedures': (0, 'polymorphism_max', r'\$[A-Za-z_]\w*|#solidify\b', ['polymorphic-call', 'overload-choice', 'contextual-overload', 'baked-record-defaults', 'operator-overloads']),
    'context-and-c-callbacks': (0, 'procedure_context_high; llvm_types_high', r'\bcontext\b|#(?:add_context|c_call)\b', ['context-threaded-call','c-callback-context']),
    'void-and-typed-pointer-arithmetic': (0, 'pointers_high; llvm_types_high', r'\b(?:input_ptr|data_tmp|dest|ptr|pointer)\s*(?:\+|-|\+=|-=)', ['pointer-offset']),
    'source-location-and-expansion-policies': (1, 'frontend_types_high; procedure_context_high; reflection_metaprogram_max', r'#(?:caller_location|caller_code|this|no_context|modify|must|discard)\b', []),
    'foreign-abi-and-library-selection': (1, 'llvm_types_high; parent trusted native dependency bridge', r'#(?:foreign|library|system_library)\b', []),
    'sdk-platform-bindings': (1, 'parent target bridge; llvm_types_high', r'\b(?:VkInstance|VkDevice|HWND|NSWindow|CFDataRef|xcb_connection_t|Display|XEvent)\b', []),
    'packed-and-explicit-record-layout': (1, 'record_layout_high; llvm_types_high', r'#(?:align|packed|pack|overlay)\b', ['record-defaults', 'union-storage']),
    'dynamic-memory-and-descriptor-lifetimes': (1, 'sequences_high; pointers_high; procedure_context_high', r'\b(?:alloc|realloc|free|array_add|array_resize|array_reserve|temporary_alloc)\s*\(', ['empty-descriptors', 'slice-alias']),
    'process-file-and-thread-effects': (1, 'parent compiler effects; procedure_context_high', r'\b(?:run_command|run_program|write_entire_file|read_entire_file|create_thread|thread_create|sleep_milliseconds)\s*\(', []),
    'atomics-and-hardware-intrinsics': (1, 'compiler_intrinsics_high; llvm_types_high', r'#intrinsic\b|\b(?:compare_and_swap|atomic_compare_exchange|atomic_add|memory_barrier|__rdtsc)\b', ['generic-intrinsic']),
    'simd-vector-lowering': (2, 'llvm_types_high; frontend_types_high', r'#(?:simd|vector)\b|\b(?:Vector4|Quaternion|float32x4|float64x2)\b', []),
}


# Parent-assigned integration ownership snapshot; update as lanes are reassigned.
ACTIVE_OWNERS = {
    'module-parameters-and-target-selection': ['module_parameters_high','generic_records_xhigh','frontend_types_high','native_target_high'],
    'compiler-workspaces-and-message-lifecycle': ['compiler_intrinsics_high','source_run_high','graph_discovery_xhigh','driver_cli_high'],
    'compiler-options-and-native-output': ['native_target_high','driver_cli_high','foreign_library_high'],
    'source-insertion-and-code-scope': ['reflection_metaprogram_max','source_run_high','procedure_calls','compiletime_conditions_high'],
    'reflection-and-type-metadata': ['reflection_metaprogram_max','type_core_review'],
    'polymorphic-and-baked-procedures': ['polymorphism_max','generic_records_xhigh'],
    'context-and-c-callbacks': ['procedure_context_high','caller_locations_high','foreign_library_high'],
    'void-and-typed-pointer-arithmetic': ['pointers_high','runtime_intrinsics_xhigh'],
    'source-location-and-expansion-policies': ['caller_locations_high','procedure_calls','polymorphism_max','reflection_metaprogram_max','optimization_hints_high'],
    'foreign-abi-and-library-selection': ['foreign_library_high','native_target_high','native_dependencies_high'],
    'sdk-platform-bindings': ['native_target_high','foreign_library_high','native_dependencies_high'],
    'packed-and-explicit-record-layout': ['storage_alignment_high','type_core_review'],
    'dynamic-memory-and-descriptor-lifetimes': ['sequences_high','pointers_high','runtime_intrinsics_xhigh','float_pipeline_high','compile_time_vm_xhigh'],
    'process-file-and-thread-effects': ['record_layout_high','os_library_acceptance_high','compile_time_io_high','source_run_high','compiler_intrinsics_high'],
    'atomics-and-hardware-intrinsics': ['runtime_intrinsics_xhigh','compiler_intrinsics_high','type_core_review'],
    'simd-vector-lowering': ['llvm_types_high','inline_types_high','type_core_review'],
}
UNOWNED = {}
EXTERNAL_CONSTRAINTS = {
    'sdk-platform-bindings': ['Actual target SDK presence, platform runtime access and graphics/window dependencies must be checked on the intended host.'],
    'foreign-abi-and-library-selection': ['Native project dependencies must be rebuilt from reviewed source using trusted tooling; supplied native artifacts remain prohibited.'],
    'process-file-and-thread-effects': ['OS-specific async behavior requires its actual target OS and APIs; source acceptance is separate.'],
    'simd-vector-lowering': ['Establish actual source vector/intrinsic use before interpreting nominal vector-name matches as hardware requirements.'],
}



def scan(root: Path = ROOT) -> dict:
    rows = {key: {'priority': priority, 'owner': '; '.join(ACTIVE_OWNERS[key]), 'pattern': pattern,
                  'curated_partial_contracts': contracts, 'occurrences': 0, 'files': 0,
                  'projects': {}, 'samples': [], 'acceptance': 'unverified full requirement', 'active_owners': ACTIVE_OWNERS[key],
                  'unowned_subrequirements': UNOWNED.get(key, []),
                  'external_constraints': EXTERNAL_CONSTRAINTS.get(key, [])}
            for key, (priority, owner, pattern, contracts) in REQUIREMENTS.items()}
    directives, compiler_names, imported_modules, libraries = {}, {}, {}, {}
    all_sources = inventory(root)
    sources = [source for source in all_sources if source.project != 'reference']
    for source in sources:
        if source.sha256 != source.expected_sha256:
            raise ValueError(f'pinned source missing or changed: {source.id}')
        raw = decode(source.path.read_bytes())
        code = mask_noncode(raw)
        newlines = [index for index, char in enumerate(code) if char == '\n']
        def record(table, key, at):
            entry = table.setdefault(key, {'occurrences':0, 'projects':{}, 'samples':[]})
            entry['occurrences'] += 1
            entry['projects'][source.project] = entry['projects'].get(source.project,0)+1
            if not any(sample['project'] == source.project for sample in entry['samples']):
                entry['samples'].append({'source':source.id, 'project':source.project,
                                         'line':bisect_right(newlines,at)+1,
                                         'sha256':source.sha256, 'revision':source.revision})
        for key, (_, _, pattern, _) in REQUIREMENTS.items():
            matches = list(re.finditer(pattern,code))
            if not matches: continue
            row = rows[key]
            row['occurrences'] += len(matches)
            row['files'] += 1
            row['projects'][source.project] = row['projects'].get(source.project,0)+1
            if not any(sample['project'] == source.project for sample in row['samples']):
                at = matches[0].start()
                row['samples'].append({'source':source.id, 'project':source.project,
                                       'line':bisect_right(newlines,at)+1,
                                       'sha256':source.sha256, 'revision':source.revision})
        for match in re.finditer(r'#[A-Za-z_]\w*',code):
            record(directives,match.group(),match.start())
            spelling = match.group()
            if spelling in ('#import','#library','#system_library'):
                # Metadata extraction only; do not retain source statements or execute them.
                literal = re.match(r'#[A-Za-z_]\w*(?:\s*,\s*[A-Za-z_]\w*)*\s*"([^"\r\n]*)"',raw[match.start():])
                if literal:
                    record(imported_modules if spelling == '#import' else libraries,
                           literal.group(1),match.start())
        for match in re.finditer(r'\bcompiler_[A-Za-z_]\w*',code):
            record(compiler_names,match.group(),match.start())
    actual_simd = None
    simd_path = root/'artifacts/simd-source-inventory.json'
    if simd_path.is_file():
        actual_simd = json.loads(simd_path.read_text())
        known = {source.path.resolve(): source for source in all_sources}
        for record in actual_simd.get('sources', []):
            path = (root/record['path']).resolve()
            source = known.get(path)
            if source is None or source.sha256 != record['sha256'] or source.sha256 != source.expected_sha256:
                raise ValueError('SIMD inventory source is not an exact pinned source')
        actual_simd = {'inventory_sha256':digest(simd_path), 'owner':'llvm_types_high',
                       'scope':actual_simd['scope'], 'sources':actual_simd['sources'],
                       'staged_instruction_subset':actual_simd['supported_instruction_plan'],
                       'target_contract':actual_simd['staged_target_contract'],
                       'remaining_source_contracts':actual_simd['remaining_source_contracts'],
                       'acceptance':'Static source evidence and staged implementation are not source-to-native acceptance.'}
    return {'format':1, 'method':'static lexical requirement evidence', 'inventory':len(sources),
            'projects':dict(Counter(source.project for source in sources)),
            'input_manifests': {name:digest(root/name) for name in ('corpus/reference-inputs.json','corpus/upstreams.json')},
            'scanner_sha256':digest(Path(__file__)), 'ownership_snapshot_date':'2026-10-02',
            'provenance_notes': {'withlang-dev/open-jai':'Alternate implementation sources and compatibility modules are not replacements for original standard-library runtime behavior; fixed-answer/no-op OS source acceptance does not establish genuine OS effects.'},
            'unowned_requirement_groups':list(UNOWNED), 'actual_simd_evidence':actual_simd, 'requirements':rows, 'directive_spellings':directives,
            'compiler_identifiers':compiler_names, 'import_targets':imported_modules, 'library_targets':libraries,
            'limitations':['Requirement priority and owner routing are engineering analysis, not compiler support observations.',
                           'Static spellings may be ambiguous; custom here strings can cause false positives.',
                           'Curated contracts cover only a small part of each requirement; full requirement remains unverified.',
                           'No original executable, object, library or project script was executed or loaded.',
                           'Target SDK/library names are metadata only; availability, ABI and complete project builds remain unverified.']}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--report',type=Path,default=ROOT/'artifacts/project-requirements.json')
    args = parser.parse_args()
    report_path = args.report.resolve()
    if any(report_path.is_relative_to(path.resolve()) for path in (ROOT/'reference',ROOT/'corpus/upstream')):
        parser.error('report must be outside source corpus')
    report = scan()
    report_path.parent.mkdir(parents=True,exist_ok=True)
    report_path.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({'inventory':report['inventory'], 'requirements':{key:row['files'] for key,row in report['requirements'].items()},
                      'directives':len(report['directive_spellings']), 'compiler_identifiers':len(report['compiler_identifiers']),
                      'import_targets':len(report['import_targets']), 'library_targets':len(report['library_targets']), 'report':str(report_path)}))
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
