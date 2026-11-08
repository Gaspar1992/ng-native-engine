import { Component, CUSTOM_ELEMENTS_SCHEMA, input } from '@angular/core';

@Component({
  selector: 'app-pokemon-card',
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  templateUrl: './pokemon-card.html',
  styleUrl: './pokemon-card.css',
})
export class PokemonCard {
  readonly name = input<string>('');
  readonly number = input<string>('');
  readonly sprite = input<string>('');
}
