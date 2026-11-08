import angularEslint from '@angular-eslint/eslint-plugin';
import angularTemplateEslint from '@angular-eslint/eslint-plugin-template';
import angularTemplateParser from '@angular-eslint/template-parser';
import tsparser from '@typescript-eslint/parser';
import { configs, processInlineTemplates } from 'angular-eslint';
import oxlint from 'eslint-plugin-oxlint';

export default [
  {
    ignores: [
      '**/dist/**',
      '**/target/**',
      '**/out-tsc/**',
      '**/.angular/**',
      '**/node_modules/**',
    ],
  },
  {
    files: ['**/*.ts'],
    processor: processInlineTemplates,
    languageOptions: {
      parser: tsparser,
    },
    plugins: {
      '@angular-eslint': angularEslint,
    },
    rules: {
      '@angular-eslint/directive-selector': [
        'error',
        { type: 'attribute', prefix: 'app', style: 'camelCase' },
      ],
      '@angular-eslint/component-selector': [
        'error',
        { type: 'element', prefix: 'app', style: 'kebab-case' },
      ],
    },
  },
  {
    files: ['**/*.html'],
    languageOptions: {
      parser: angularTemplateParser,
    },
    plugins: {
      '@angular-eslint/template': angularTemplateEslint,
    },
    rules: {
      ...configs.templateRecommended[1].rules,
    },
  },
  ...oxlint.buildFromOxlintConfigFile('./.oxlintrc.json'),
];
