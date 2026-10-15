import hashlib
import json
from pathlib import Path
import tempfile
import shutil
import unittest
from unittest.mock import patch
import sdl3_native as sdl
import build_native_libs as pipeline

class BuildContract(unittest.TestCase):
    def test_six_platforms_are_explicit(self):
        self.assertEqual(len(sdl.PLATFORMS),6)
        for p in sdl.PLATFORMS:
            args=sdl.configure_options(p)
            for opt in ('AUDIO','VIDEO','EVENTS','JOYSTICK','RENDER','GPU'):
                self.assertIn(f'-DSDL_{opt}=ON',args)
            self.assertIn('-DSDL_STATIC=ON',args)
            self.assertIn('-DSDL_SHARED=ON',args)
    def test_architectures_are_not_build_host_defaults(self):
        self.assertIn('-DCMAKE_OSX_ARCHITECTURES=x86_64',sdl.configure_options('macos-x64'))
        self.assertIn('-DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded',sdl.configure_options('windows-arm64'))
        with self.assertRaises(ValueError):sdl.configure_options('linux-riscv64')
    def test_windows_does_not_assume_vs2022(self):
        self.assertNotIn('Visual Studio 17 2022',sdl.configure_options('windows-x64'))
    def test_unbound_static_dependencies_fail(self):
        sdl.check_dependencies('$<LINK_ONLY:m>;$<LINK_ONLY:$<LINK_LIBRARY:WEAK_FRAMEWORK,CoreHaptics>>;$<TARGET_NAME:SDL3::Headers>;\n-lpthread','macos-arm64')
        with self.assertRaisesRegex(RuntimeError,'unbound transitive'):
            sdl.check_dependencies('$<LINK_ONLY:PkgConfig::UNEXPECTED>','linux-x64')
    def test_bad_cached_archive_is_rejected_offline(self):
        with tempfile.TemporaryDirectory(dir=sdl.ROOT/'artifacts/sdl3/tmp') as d:
            p=Path(d);(p/'SDL3-3.4.18.tar.gz').write_bytes(b'corrupt')
            with self.assertRaisesRegex(RuntimeError,'cache hash mismatch'):sdl.checked_archive(p,True)
    def test_offline_missing_archive_is_rejected(self):
        with tempfile.TemporaryDirectory(dir=sdl.ROOT/'artifacts/sdl3/tmp') as d:
            with self.assertRaisesRegex(RuntimeError,'offline: missing'):sdl.checked_archive(Path(d),True)
    def test_package_inventory_includes_import_license_and_metadata(self):
        manifest=json.loads(pipeline.MANIFEST.read_text())
        for p in sdl.PLATFORMS:
            files=pipeline.shipped_files(manifest,p)
            self.assertIn('licenses/sdl3/LICENSE.txt',files)
            self.assertIn('sdl3/build.json',files)
            if p.startswith('windows'):self.assertIn('SDL3-import.lib',files)

class InstalledSDK(unittest.TestCase):
    def fixture(self,root,relative):
        stage=root/'install';directory=stage/relative;directory.mkdir(parents=True)
        # Layout/copy fixtures only: these are not native artifacts or functional
        # CMake packages. Real exported-target validation is the native consumer gate.
        for name in ('SDL3Config.cmake','SDL3ConfigVersion.cmake','SDL3headersTargets.cmake',
                     'SDL3staticTargets.cmake','SDL3staticTargets-release.cmake',
                     'SDL3sharedTargets.cmake','SDL3sharedTargets-release.cmake'):
            (directory/name).write_bytes(b'# installed target metadata fixture\n')
        header=stage/'include/SDL3/SDL.h';header.parent.mkdir(parents=True);header.write_bytes(b'/* header copy fixture */\n')
        for name in ('lib/SDL3-static.lib','lib/SDL3.lib','bin/SDL3.dll'):
            path=stage/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(name.encode())
        manifest=root/'install_manifest.txt'
        manifest.write_text(''.join(p.as_posix()+'\n' for p in stage.rglob('*') if p.is_file()))
        return stage,directory,manifest

    def test_actual_install_manifest_selects_windows_unix_and_custom_layout(self):
        for relative in ('cmake','lib/cmake/SDL3','custom/export/SDL3'):
            with tempfile.TemporaryDirectory(dir=sdl.ROOT/'artifacts/sdl3/tmp') as d:
                stage,directory,manifest=self.fixture(Path(d),relative)
                self.assertEqual(sdl.installed_cmake_dir(stage,manifest),directory)

    def test_missing_ambiguous_or_external_exports_fail(self):
        with tempfile.TemporaryDirectory(dir=sdl.ROOT/'artifacts/sdl3/tmp') as d:
            root=Path(d);stage,directory,manifest=self.fixture(root,'cmake')
            export=directory/'SDL3sharedTargets-release.cmake';export.unlink()
            with self.assertRaisesRegex(RuntimeError,'missing installed CMake'):sdl.installed_cmake_dir(stage,manifest)
            export.write_text('# restored layout fixture\n')
            manifest.write_text(manifest.read_text()+str(root/'other/SDL3Config.cmake')+'\n')
            with self.assertRaisesRegex(RuntimeError,'exactly one'):sdl.installed_cmake_dir(stage,manifest)
            manifest.write_text(str(root/'other/SDL3Config.cmake')+'\n')
            with self.assertRaisesRegex(RuntimeError,'outside'):sdl.installed_cmake_dir(stage,manifest)

    def test_sdk_relocation_preserves_installed_tree_and_artifact_identity(self):
        with tempfile.TemporaryDirectory(dir=sdl.ROOT/'artifacts/sdl3/tmp') as d:
            root=Path(d);stage,directory,manifest=self.fixture(root,'cmake')
            package=root/'package';package.mkdir()
            info=sdl.package_installed_sdk(stage,package,manifest)
            relocated=root/'relocated package';shutil.move(package,relocated)
            self.assertEqual(info['prefix'],'sdl3/sdk');self.assertEqual(info['cmake_dir'],'cmake')
            sdk=relocated/info['prefix']
            for name,digest in info['files'].items():
                self.assertEqual(hashlib.sha256((sdk/name).read_bytes()).hexdigest(),digest)
                self.assertEqual((sdk/name).read_bytes(),(stage/name).read_bytes())
            self.assertEqual((sdk/'lib/SDL3-static.lib').read_bytes(),b'lib/SDL3-static.lib')
            self.assertEqual((sdk/'lib/SDL3.lib').read_bytes(),b'lib/SDL3.lib')
            self.assertEqual((sdk/'bin/SDL3.dll').read_bytes(),b'bin/SDL3.dll')
            self.assertTrue((sdk/'cmake/SDL3Config.cmake').is_file())

class WindowsDiscovery(unittest.TestCase):
    instances=[
        {'installationPath':'C:/VS/2022','installationVersion':'17.14.1.0','isComplete':True,'isLaunchable':True},
        {'installationPath':'C:/VS/2026','installationVersion':'18.1.2.0','isComplete':True,'isLaunchable':True},
    ]
    generators=['Visual Studio 17 2022','Visual Studio 18 2026','Ninja']

    def select(self,plat='windows-x64',env=None,instances=None,generators=None):
        return sdl.select_windows_toolchain(self.instances if instances is None else instances,
                                            self.generators if generators is None else generators,plat,env or {})

    def test_latest_supported_real_install_and_explicit_architecture(self):
        for plat,cpu in [('windows-x64','x64'),('windows-arm64','ARM64')]:
            args=self.select(plat)
            self.assertEqual(args,['-G','Visual Studio 18 2026','-A',cpu,
                                   '-T',f'host={cpu}','-DCMAKE_GENERATOR_INSTANCE=C:/VS/2026'])

    def test_older_cmake_selects_supported_install_not_nonexistent_vs(self):
        args=self.select(generators=['Visual Studio 17 2022','Ninja'])
        self.assertIn('-DCMAKE_GENERATOR_INSTANCE=C:/VS/2022',args)
        with self.assertRaisesRegex(RuntimeError,'No compatible'):
            self.select(instances=[self.instances[1]],generators=['Visual Studio 17 2022'])

    def test_incomplete_and_unlaunchable_instances_are_rejected(self):
        for flag in ('isComplete','isLaunchable'):
            with self.assertRaisesRegex(RuntimeError,'No compatible'):
                self.select(instances=[dict(self.instances[1],**{flag:False})])
        with self.assertRaisesRegex(RuntimeError,'No compatible'):self.select(instances=[])

    def test_explicit_environment_selects_only_verified_install(self):
        self.assertIn('Visual Studio 17 2022',self.select(env={'CMAKE_GENERATOR':'Visual Studio 17 2022'}))
        self.assertIn('Visual Studio 17 2022',self.select(env={'CMAKE_GENERATOR_INSTANCE':r'c:\VS\2022'}))
        self.assertIn('Visual Studio 17 2022',self.select(env={'VSINSTALLDIR':'C:/VS/2022/'}))
        for env in ({'CMAKE_GENERATOR':'Ninja'},{'CMAKE_GENERATOR_INSTANCE':'C:/missing'},
                    {'CMAKE_GENERATOR_PLATFORM':'ARM64'},{'CMAKE_GENERATOR_TOOLSET':'host=x86'}):
            with self.assertRaises(RuntimeError):self.select(env=env)

    def test_discovery_queries_target_tools_and_available_cmake_generators(self):
        for plat,component in [('windows-x64','x86.x64'),('windows-arm64','ARM64')]:
            with patch.object(sdl.shutil,'which',return_value='vswhere.exe'), \
                 patch.object(sdl.subprocess,'run') as run:
                run.side_effect=[type('Result',(),{'stdout':json.dumps(self.instances)})(),
                                 type('Result',(),{'stdout':json.dumps({'generators':[{'name':g} for g in self.generators]})})()]
                self.assertIn('Visual Studio 18 2026',sdl.windows_toolchain(plat,{}))
                query=run.call_args_list[0].args[0]
                self.assertIn(f'Microsoft.VisualStudio.Component.VC.Tools.{component}',query)
                self.assertIn('Microsoft.Component.MSBuild',query)
                self.assertIn('-utf8',query)
                self.assertEqual(run.call_args_list[1].args[0],['cmake','-E','capabilities'])

    def test_missing_discovery_tool_is_actionable(self):
        with patch.object(sdl.shutil,'which',return_value=None):
            with self.assertRaisesRegex(RuntimeError,'vswhere'):sdl.windows_toolchain('windows-x64',{})

    def test_generator_change_discards_only_owned_cmake_build_tree(self):
        with tempfile.TemporaryDirectory(dir=sdl.ROOT/'artifacts/sdl3/tmp') as d:
            root=Path(d);build=root/'build';build.mkdir();sentinel=root/'source';sentinel.write_text('keep')
            cache=build/'CMakeCache.txt'
            cache.write_text('CMAKE_GENERATOR:INTERNAL=Visual Studio 17 2022\nCMAKE_GENERATOR_INSTANCE:INTERNAL=C:/VS/2022\nCMAKE_GENERATOR_PLATFORM:INTERNAL=x64\nCMAKE_GENERATOR_TOOLSET:INTERNAL=host=x64\n')
            args=self.select(env={'CMAKE_GENERATOR':'Visual Studio 17 2022'})
            sdl.prepare_windows_build_tree(build,args)
            self.assertTrue(cache.exists())
            sdl.prepare_windows_build_tree(build,self.select())
            self.assertFalse(build.exists());self.assertEqual(sentinel.read_text(),'keep')

    def test_packaging_preserves_static_import_and_runtime_identity(self):
        with tempfile.TemporaryDirectory(dir=sdl.ROOT/'artifacts/sdl3/tmp') as d:
            root=Path(d);stage=root/'install';out=root/'package';out.mkdir()
            # Distinct sentinel bytes test copy routing, not native Windows behavior.
            artifacts={'lib/SDL3-static.lib':b'static archive','lib/SDL3.lib':b'DLL import library','bin/SDL3.dll':b'runtime'}
            for name,data in artifacts.items():
                path=stage/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(data)
            self.assertEqual(set(sdl.package_windows_artifacts(stage,out)),{'SDL3.lib','SDL3-import.lib','SDL3.dll'})
            self.assertEqual((out/'SDL3.lib').read_bytes(),artifacts['lib/SDL3-static.lib'])
            self.assertEqual((out/'SDL3-import.lib').read_bytes(),artifacts['lib/SDL3.lib'])
            self.assertEqual((out/'SDL3.dll').read_bytes(),artifacts['bin/SDL3.dll'])
            (stage/'lib/SDL3.lib').write_bytes(artifacts['lib/SDL3-static.lib'])
            with self.assertRaisesRegex(RuntimeError,'identical'):sdl.package_windows_artifacts(stage,out)

if __name__=='__main__':unittest.main()
