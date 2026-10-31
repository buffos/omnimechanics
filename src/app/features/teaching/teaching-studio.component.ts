import { ChangeDetectionStrategy, Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TeachingSidebarComponent } from './teaching-sidebar/teaching-sidebar.component';
import { TeachingCanvasComponent } from './teaching-canvas/teaching-canvas.component';
import { StepWalkerComponent } from './step-walker/step-walker.component';

@Component({
  selector: 'app-teaching-studio',
  standalone: true,
  imports: [
    CommonModule,
    TeachingSidebarComponent,
    TeachingCanvasComponent,
    StepWalkerComponent,
  ],
  templateUrl: './teaching-studio.component.html',
  styleUrls: ['./teaching-studio.component.css'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class TeachingStudioComponent {}
