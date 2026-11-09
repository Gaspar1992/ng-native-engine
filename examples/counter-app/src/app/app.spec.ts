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

  it('should initialize default signals and navigate between pokedex list and detail views', () => {
    const fixture = TestBed.createComponent(App);
    const app = fixture.componentInstance;

    expect(app['windowTitle']()).toBe('Pokédex Nacional');
    expect(app['subtitle']()).toBe('Angular 22 + GTK4 / Libadwaita');
    expect(app['author']()).toBe('ng-native-dev');
    expect(app['pokemonId']()).toBe(0);

    app.selectBulbasaur();
    expect(app['pokemonId']()).toBe(1);

    app.selectPikachu();
    expect(app['pokemonId']()).toBe(25);

    app.backToList();
    expect(app['pokemonId']()).toBe(0);

    app.nextPokemon();
    expect(app['pokemonId']()).toBe(1);

    app.previousPokemon();
    expect(app['pokemonId']()).toBe(0);

    app.increment();
    app.increment();
    expect(app['pokemonId']()).toBe(2);

    app.decrement();
    expect(app['pokemonId']()).toBe(1);

    app.reset();
    expect(app['pokemonId']()).toBe(0);
  });
});
