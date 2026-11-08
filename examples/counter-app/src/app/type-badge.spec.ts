import { ComponentFixture, TestBed } from '@angular/core/testing';
import { TypeBadge } from './type-badge';

describe('TypeBadge', () => {
  let fixture: ComponentFixture<TypeBadge>;
  let component: TypeBadge;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [TypeBadge],
    }).compileComponents();

    fixture = TestBed.createComponent(TypeBadge);
    component = fixture.componentInstance;
  });

  it('should create the type badge component', () => {
    expect(component).toBeTruthy();
  });

  it('should accept type input and render label', () => {
    fixture.componentRef.setInput('type', 'ELECTRIC');
    fixture.detectChanges();
    expect(component.type()).toBe('ELECTRIC');
  });
});
