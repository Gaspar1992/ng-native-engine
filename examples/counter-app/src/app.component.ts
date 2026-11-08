// Cross-Platform Angular Native Component (Linux GTK4 / Windows WinUI / macOS AppKit)
export class AppComponent {
  windowTitle = 'NgNative - Hola Mundo';
  greeting = 'Hola Mundo desde Angular Native!';
  count = 0;

  static styles = `
    .main-card {
      background: #242424;
      color: #ffffff;
      padding: 16;
      spacing: 12;
    }
    .hero-title {
      background: none;
      color: #78aeed;
      padding: 8;
      spacing: 4;
    }
    .suggested-action {
      background: #3584e4;
      color: #ffffff;
      padding: 12;
      spacing: 8;
    }
  `;

  static template = `
    <ng-window id="win" title="{{ windowTitle }}">
      <ng-box id="vbox" class="main-card">
        <ng-label id="hello" class="hero-title">{{ greeting }}</ng-label>
        <ng-label id="lbl">Count: {{ count }}</ng-label>
        <ng-button id="btn" class="suggested-action">Increment</ng-button>
      </ng-box>
    </ng-window>
  `;
}
