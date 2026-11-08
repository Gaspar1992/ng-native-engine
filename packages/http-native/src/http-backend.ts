import {
  HttpBackend,
  HttpErrorResponse,
  HttpEvent,
  HttpFeature,
  HttpFeatureKind,
  HttpHeaders,
  HttpRequest,
  HttpResponse,
  provideHttpClient,
} from '@angular/common/http';
import { EnvironmentProviders, Injectable, makeEnvironmentProviders } from '@angular/core';
import { ensureNgNativeBridge } from '@ng-native/platform';
import { Observable } from 'rxjs';

/**
 * Custom Angular `HttpBackend` that bypasses browser `XMLHttpRequest` / `fetch`
 * and executes requests through `globalThis.__NG_NATIVE__.httpRequest` in the Rust engine.
 * Compatible with `HttpClient`, functional `HttpInterceptorFn`, and `httpResource()`.
 */
@Injectable({ providedIn: 'root' })
export class NativeHttpBackend implements HttpBackend {
  handle(req: HttpRequest<unknown>): Observable<HttpEvent<unknown>> {
    return new Observable<HttpEvent<unknown>>((observer) => {
      try {
        const bridge = ensureNgNativeBridge();
        const serializedBody = req.serializeBody();
        const bodyStr =
          serializedBody === null || serializedBody === undefined
            ? undefined
            : typeof serializedBody === 'string'
              ? serializedBody
              : JSON.stringify(serializedBody);

        const rawRes = bridge.httpRequest(req.method, req.urlWithParams, bodyStr);
        let parsedBody: unknown = rawRes.body;
        if (req.responseType === 'json' && rawRes.body.length > 0) {
          try {
            parsedBody = JSON.parse(rawRes.body);
          } catch {
            parsedBody = rawRes.body;
          }
        }

        if (rawRes.status >= 200 && rawRes.status < 300) {
          observer.next(
            new HttpResponse({
              body: parsedBody,
              headers: new HttpHeaders(),
              status: rawRes.status,
              statusText: rawRes.statusText,
              url: req.urlWithParams,
            }),
          );
          observer.complete();
        } else {
          observer.error(
            new HttpErrorResponse({
              error: parsedBody,
              headers: new HttpHeaders(),
              status: rawRes.status,
              statusText: rawRes.statusText,
              url: req.urlWithParams,
            }),
          );
        }
      } catch (err) {
        observer.error(err);
      }
    });
  }
}

/**
 * Configures Angular `HttpClient` to use `NativeHttpBackend` over the Rust host bridge.
 */
export function provideNativeHttpClient(
  ...features: HttpFeature<HttpFeatureKind>[]
): EnvironmentProviders {
  return makeEnvironmentProviders([
    provideHttpClient(...features),
    { provide: HttpBackend, useExisting: NativeHttpBackend },
  ]);
}
