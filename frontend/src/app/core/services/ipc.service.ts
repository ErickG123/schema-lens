import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { SaveFileResult } from '../models/ipc.model';

@Injectable({
  providedIn: 'root'
})
export class TauriIpcService {
  async invoke<T>(command: string, args: Record<string, any> = {}): Promise<T> {
    if (typeof window === 'undefined' || !(window as any).__TAURI_INTERNALS__) {
      throw new Error(`Tauri command '${command}' is unavailable outside the desktop runtime.`);
    }
    return await invoke<T>(command, args);
  }
  async testConnection(payload: any): Promise<any> {
    return await this.invoke('test_connection', { payload });
  }

  async startComparison(source: any, target: any): Promise<any> {
    return await this.invoke('run_comparison', { source, target });
  }

  async generateSql(): Promise<string> {
    return await this.invoke('generate_migration_sql');
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
