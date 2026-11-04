export async function resolve(specifier, context, next) {
  if (specifier.startsWith(".") && specifier.endsWith(".js")) {
    try {
      return await next(specifier.slice(0, -3) + ".ts", context);
    } catch {
      /* fall through to the original specifier */
    }
  }
  return next(specifier, context);
}
