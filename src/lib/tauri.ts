import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  Book,
  Chapter,
  ProgressEvent,
  Settings,
  SettingsUpdate,
  Source,
  TestProviderResult,
} from "./types";

export async function startBook(title: string): Promise<Book> {
  return invoke<Book>("start_book", { title });
}

export async function listBooks(): Promise<Book[]> {
  return invoke<Book[]>("list_books");
}

export async function getBook(id: number): Promise<Book> {
  return invoke<Book>("get_book", { id });
}

export async function getSources(bookId: number): Promise<Source[]> {
  return invoke<Source[]>("get_sources", { bookId });
}

export async function getChapters(bookId: number): Promise<Chapter[]> {
  return invoke<Chapter[]>("get_chapters", { bookId });
}

export async function getChapterSources(chapterId: number): Promise<Source[]> {
  return invoke<Source[]>("get_chapter_sources", { chapterId });
}

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export async function updateSettings(patch: SettingsUpdate): Promise<Settings> {
  return invoke<Settings>("update_settings", { patch });
}

export async function testProvider(): Promise<TestProviderResult> {
  return invoke<TestProviderResult>("test_provider");
}

export function onProgress(
  handler: (e: ProgressEvent) => void,
): Promise<UnlistenFn> {
  return listen<ProgressEvent>("book.progress", (evt) => handler(evt.payload));
}

export function onBookReady(
  handler: (bookId: number) => void,
): Promise<UnlistenFn> {
  return listen<{ book_id: number }>("book.ready", (evt) =>
    handler(evt.payload.book_id),
  );
}

export function onBookFailed(
  handler: (payload: { book_id: number; error: string }) => void,
): Promise<UnlistenFn> {
  return listen<{ book_id: number; error: string }>("book.failed", (evt) =>
    handler(evt.payload),
  );
}
