// Hand-drawn pass for readme_banner.py. Reads {paths: [d, ...], options} from
// stdin, prints [{fill: [d], stroke: [d]}] for each path. Needs `roughjs`
// resolvable from the working directory (readme_banner.py installs it in a
// temporary directory); nothing shipped depends on this file.
import rough from 'roughjs/bundled/rough.esm.js';

const input: { paths: string[]; options: { seed: number } & Record<string, unknown> } = JSON.parse(
  await new Promise<string>((done) => {
    let text = '';
    process.stdin.on('data', (chunk) => (text += chunk));
    process.stdin.on('end', () => done(text));
  }),
);

const gen = rough.generator();
const out = input.paths.map((d: string, i: number) => {
  const drawable = gen.path(d, {
    ...input.options,
    seed: input.options.seed + i,
    fill: 'x',
    fillStyle: 'solid',
  });
  const parts = gen.toPaths(drawable);
  return {
    fill: parts.filter((p) => p.fill && p.fill !== 'none').map((p) => p.d),
    stroke: parts
      .filter((p) => p.stroke && p.stroke !== 'none' && (!p.fill || p.fill === 'none'))
      .map((p) => p.d),
  };
});
process.stdout.write(JSON.stringify(out));
