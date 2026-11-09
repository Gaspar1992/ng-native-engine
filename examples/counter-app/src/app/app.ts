import { Component, CUSTOM_ELEMENTS_SCHEMA, signal } from '@angular/core';
import { PokemonCard } from './pokemon-card';
import { TypeBadge } from './type-badge';

@Component({
  selector: 'app-root',
  imports: [PokemonCard, TypeBadge],
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  templateUrl: './app.html',
  styleUrl: './app.css',
})
export class App {
  protected readonly windowTitle = signal('Pokédex Nacional');
  protected readonly subtitle = signal('Angular 22 + GTK4 / Libadwaita');
  protected readonly greeting = signal('Hola Mundo desde Angular Native!');
  protected readonly author = signal('ng-native-dev');
  protected readonly pokemonId = signal(0);

  increment(): void {
    this.pokemonId.update((value) => value + 1);
  }

  decrement(): void {
    this.pokemonId.update((value) => value - 1);
  }

  reset(): void {
    this.pokemonId.set(0);
  }

  nextPokemon(): void {
    this.pokemonId.update((value) => value + 1);
  }

  previousPokemon(): void {
    this.pokemonId.update((value) => value - 1);
  }

  backToList(): void {
    this.pokemonId.set(0);
  }

  selectBulbasaur(): void {
    this.pokemonId.set(1);
  }

  selectCharmander(): void {
    this.pokemonId.set(4);
  }

  selectSquirtle(): void {
    this.pokemonId.set(7);
  }

  selectPikachu(): void {
    this.pokemonId.set(25);
  }

  selectCharizard(): void {
    this.pokemonId.set(6);
  }

  selectGengar(): void {
    this.pokemonId.set(94);
  }
}
