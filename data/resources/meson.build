# Resources
blueprints = custom_target(
  'blueprints',
  input: files(
    'ui/content-pages/ability.blp',
    'ui/content-pages/move.blp',
    'ui/content-pages/pokemon.blp',
    'ui/shortcuts.blp',
    'ui/window.blp',
  ),
  output: '.',
  command: [
    find_program('blueprint-compiler'),
    'batch-compile',
    '@OUTPUT@',
    '@CURRENT_SOURCE_DIR@',
    '@INPUT@',
  ],
)

resources = gnome.compile_resources(
  'resources',
  'resources.gresource.xml',
  gresource_bundle: true,
  source_dir: meson.current_build_dir(),
  install: true,
  install_dir: pkgdatadir,
  dependencies: blueprints,
)
