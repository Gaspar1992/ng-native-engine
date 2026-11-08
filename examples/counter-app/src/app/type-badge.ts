import { Component, CUSTOM_ELEMENTS_SCHEMA, input } from '@angular/core';

@Component({
  selector: 'app-type-badge',
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  templateUrl: './type-badge.html',
  styleUrl: './type-badge.css',
})
export class TypeBadge {
  readonly type = input<string>('');
}
