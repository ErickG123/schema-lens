import { Injectable, signal } from '@angular/core';
import { ConfirmDialogOptions, ToastMessage, ToastType } from '../models/notification.model';

@Injectable({
  providedIn: 'root'
})
export class NotificationService {
  toasts = signal<ToastMessage[]>([]);
  activeConfirm = signal<{
    options: ConfirmDialogOptions;
    resolve: (value: boolean) => void;
  } | null>(null);

  show(type: ToastType, title: string, message: string, duration = 5000): void {
    const id = Math.random().toString(36).substring(2, 9);
    const toast: ToastMessage = { id, type, title, message, duration };

    this.toasts.update(current => [...current, toast]);

    if (duration > 0) {
      setTimeout(() => {
        this.dismiss(id);
      }, duration);
    }
  }

  success(title: string, message: string, duration = 4000): void {
    this.show('success', title, message, duration);
  }

  error(title: string, message: string, duration = 7000): void {
    this.show('error', title, message, duration);
  }

  warning(title: string, message: string, duration = 5000): void {
    this.show('warning', title, message, duration);
  }

  info(title: string, message: string, duration = 4000): void {
    this.show('info', title, message, duration);
  }

  dismiss(id: string): void {
    this.toasts.update(current => current.filter(t => t.id !== id));
  }

  confirm(options: ConfirmDialogOptions): Promise<boolean> {
    return new Promise(resolve => {
      this.activeConfirm.set({ options, resolve });
    });
  }

  resolveConfirm(confirmed: boolean): void {
    const active = this.activeConfirm();
    if (active) {
      active.resolve(confirmed);
      this.activeConfirm.set(null);
    }
  }
}
