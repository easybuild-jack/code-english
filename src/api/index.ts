import { invoke } from "@tauri-apps/api/core";
import type {
  AskResponse,
  ChatMessage,
  Favorite,
  HistoryItem,
  LookupResult,
  NewFavorite,
  Profile,
  ProviderConfig,
  ProviderView,
  Scene,
  TranslateResponse,
} from "./types";

export * from "./types";

export const api = {
  translate: (source: string, scene: Scene, providerId?: string) =>
    invoke<TranslateResponse>("translate", { source, scene, providerId }),
  ask: (messages: ChatMessage[], scene: Scene, providerId?: string) =>
    invoke<AskResponse>("ask", { messages, scene, providerId }),
  lookup: (text: string, context: string, source: string, scene: Scene, accent: "us" | "uk", providerId?: string) =>
    invoke<LookupResult>("lookup_word", { text, context, source, scene, accent, providerId }),

  getProfile: () => invoke<Profile>("get_profile"),
  saveProfile: (profile: Profile) => invoke<void>("save_profile", { profile }),

  listProviders: () => invoke<ProviderView[]>("list_providers"),
  saveProvider: (config: ProviderConfig, apiKey?: string) =>
    invoke<void>("save_provider", { config, apiKey }),
  testProvider: (providerId: string) => invoke<number>("test_provider", { providerId }),

  listHistory: (query?: string, limit = 200, offset = 0) =>
    invoke<HistoryItem[]>("list_history", { query, limit, offset }),
  deleteHistory: (id: number) => invoke<void>("delete_history", { id }),
  clearHistory: () => invoke<void>("clear_history"),

  addFavorite: (favorite: NewFavorite) =>
    invoke<{ id: number; created: boolean }>("add_favorite", { favorite }),
  listFavorites: () => invoke<Favorite[]>("list_favorites"),
  deleteFavorite: (id: number) => invoke<void>("delete_favorite", { id }),
  favoriteLookup: () => invoke<[string, number][]>("favorite_lookup"),
  saveSyncToken: (token: string) => invoke<void>("save_sync_token", { token }),
  hasSyncToken: () => invoke<boolean>("has_sync_token"),
  syncFavorites: (baseUrl: string) => invoke<void>("sync_favorites", { baseUrl }),

  dataDir: () => invoke<string>("data_dir"),
  exportData: () => invoke<string>("export_data"),
};
