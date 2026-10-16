// The gallery imports `nsh/clipboard`, which this runtime does not have
// yet: copying only shows its "copied" state here.
export const Clipboard = {
  setString(_text: string): void {},
};
