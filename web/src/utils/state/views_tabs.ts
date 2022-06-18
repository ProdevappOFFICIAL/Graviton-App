import { atom } from "recoil";
import { Tab } from "../../modules/tab";
import { clientState } from "../state";
import getAllStateData from "../state_data";

export interface ViewPanel<T> {
  selected_tab_id?: string;
  tabs: Array<T>;
  id: string;
}

export interface Views<T> {
  view_panels: Array<ViewPanel<T>>;
  id: string;
}

export function newId() {
  return Math.random().toString();
}

export function newEmptyView(): Views<Tab> {
  return {
    id: newId(),
    view_panels: [
      {
        id: newId(),
        tabs: [],
      },
    ],
  };
}

/*
 * Views and tabs openeds
 */
export const openedViewsAndTabs = atom({
  key: "openedViewsAndTabs",
  default: [newEmptyView()] as Array<Views<Tab>>,
  dangerouslyAllowMutability: true,
  effects_UNSTABLE: [
    ({ onSet, getLoadable }) => {
      onSet(() => {
        const data = getAllStateData(
          getLoadable(openedViewsAndTabs).getValue(),
        );
        const client = getLoadable(clientState).getValue();
        client.set_state_by_id(data);
      });
    },
  ],
});