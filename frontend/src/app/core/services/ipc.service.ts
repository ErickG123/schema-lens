import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { SaveFileResult } from '../models/ipc.model';

@Injectable({
  providedIn: 'root'
})
export class TauriIpcService {
  async invoke<T>(command: string, args: Record<string, any> = {}): Promise<T> {
    try {
      if (typeof window === 'undefined' || !(window as any).__TAURI_INTERNALS__) {
        console.warn(`[Tauri Fallback] Invoke '${command}' called outside Tauri runtime.`);
        return {} as T;
      }
      return await invoke<T>(command, args);
    } catch (err: any) {
      if (typeof window === 'undefined' || !(window as any).__TAURI_INTERNALS__) {
        console.warn(`[Tauri Fallback] Invoke '${command}' error outside runtime:`, err);
        return {} as T;
      }
      throw err;
    }
  }

  async testConnection(payload: any): Promise<any> {
    return await this.invoke('test_connection', { payload });
  }

  async startComparison(source: any, target: any): Promise<any> {
    return await this.invoke('run_comparison', { source, target });
  }

  async generateSql(targetDatabase: string, diffIds: string[] = []): Promise<string> {
    return await this.invoke('generate_migration_sql', { targetDatabase, diffIds });
  }

  async saveFile(path: string, content: string): Promise<SaveFileResult> {
    return await this.invoke('save_exported_file', { path, content });
  }

  async promptSaveFilePath(defaultFileName: string, filterName: string, extension: string): Promise<string | null> {
    try {
      const selected = await save({
        title: `Salvar ${filterName}`,
        defaultPath: defaultFileName,
        filters: [{
          name: filterName,
          extensions: [extension]
        }]
      });

      return selected;
    } catch (err) {
      console.warn('[Dialog Error] Falha ao abrir diálogo nativo via plugin:', err);
      return null;
    }
  }
}
