import { Component, HostListener, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { NotificationService } from '../services/notification.service';

@Component({
  selector: 'app-confirm-dialog',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './confirm-dialog.component.html',
  styleUrl: './confirm-dialog.component.css'
})
export class ConfirmDialogComponent {
  notificationService = inject(NotificationService);

  @HostListener('window:keydown.escape')
  onEscape(): void {
    if (this.notificationService.activeConfirm()) {
      this.onCancel();
    }
  }

  onConfirm(): void {
    this.notificationService.resolveConfirm(true);
  }

  onCancel(): void {
    this.notificationService.resolveConfirm(false);
  }
}
