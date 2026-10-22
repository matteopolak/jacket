/** Map editor stdlib settings to the environment shared by jailsp and jaic. */
export function toolEnvironment(
  get: (setting: string) => string | undefined,
  inherited: NodeJS.ProcessEnv,
  home: string,
): Record<string, string> {
  const env: Record<string, string> = {};
  for (const [key, value] of Object.entries(inherited)) if (value !== undefined) env[key] = value;
  for (const [setting, variable] of [["stdlib.path", "JAIC_STDLIB"], ["stdlib.prelude", "JAIC_PRELUDE"]]) {
    const value = get(setting)?.trim();
    if (value) env[variable] = value.replace(/^~(?=$|[\\/])/, home);
  }
  return env;
}
