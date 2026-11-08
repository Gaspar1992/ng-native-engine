import { TestBed } from '@angular/core/testing';
import { App } from './app';

describe('App', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [App],
    }).compileComponents();
  });

  it('should create the app', () => {
    const fixture = TestBed.createComponent(App);
    const app = fixture.componentInstance;
    expect(app).toBeTruthy();
  });

  it('should initialize default signals and increment counter reactively', () => {
    const fixture = TestBed.createComponent(App);
    const app = fixture.componentInstance;

    expect(app['windowTitle']()).toBe('NgNative - Hola Mundo');
    expect(app['subtitle']()).toBe('Angular 22 + GTK4 / Libadwaita');
    expect(app['author']()).toBe('ng-native-dev');
    expect(app['count']()).toBe(0);

    app.increment();
    app.increment();
    expect(app['count']()).toBe(2);

    app.decrement();
    expect(app['count']()).toBe(1);

    app.reset();
    expect(app['count']()).toBe(0);
  });
});
