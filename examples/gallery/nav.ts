import { type JSX } from "nsh";
import { createNavigator } from "nsh/ui/mobile";
import type { IconName } from "./icons";

/** A demo of the index. */
export interface DemoEntry {
  name: string;
  detail: string;
  icon: IconName;
  screen: () => JSX.Element;
}

/** A screen a demo pushes: a component rendering its own `<Screen>`. */
export type Route = { name: "home" } | { name: "demo"; demo: DemoEntry } | { name: "page"; page: () => JSX.Element };

export const nav = createNavigator<Route>({ name: "home" });
