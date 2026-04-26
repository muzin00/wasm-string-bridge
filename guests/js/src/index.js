export function processString(input) {
  return input.replace(/[a-z]/g, (c) => String.fromCharCode(c.charCodeAt(0) - 32));
}
