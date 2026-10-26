# Memoria de trabajo para agentes: XuperCli

Última actualización: 2026-10-26. Este archivo reúne el estado del repositorio y los hallazgos de ingeniería inversa de la app Xuper/MagisTV. Distingue lo observado de lo que sigue siendo una hipótesis. Revalida los detalles de red antes de convertirlos en código: los dominios, versiones y enlaces pueden cambiar.

## Encargo y alcance

- El usuario quiere un `ani-cli` para Xuper/MagisTV escrito en Rust. La prioridad actual es Linux; Windows queda para después de una versión estable.
- La experiencia deseada es buscar programas en una lista dentro de la terminal, escoger uno y reproducirlo con `mpv` o VLC. Por ahora el código usa `mpv`.
- El objetivo técnico pendiente es descubrir y utilizar el catálogo y la obtención de enlaces de reproducción de la app real, sin requerir que el usuario suministre una lista M3U o una cuenta. El usuario indicó que la app Android permite navegar y reproducir sin iniciar sesión. Eso demuestra acceso invitado en la app, pero **no demuestra** que una petición HTTP aislada funcione sin los metadatos, tokens de dispositivo o firmas que genera la app.
- El usuario autorizó anteriormente instalar la actualización 4.99.20 y realizar una captura filtrada con PCAPdroid en su teléfono. El trabajo de esta sesión documentó el resultado; no infieras que el teléfono sigue conectado o conserva el mismo estado.
- En este proyecto, no afirmes que el CLI ya habla con Xuper: **todavía no hay integración de catálogo ni reproducción de Xuper**. Los modos disponibles trabajan con fuentes externas explícitas.

## Estado del repositorio (2026-10-26)

- Rama observada: `agent/init`, `HEAD` `7044fab` (`Working on find correct domain`); `feature/getData` apuntaba al mismo commit. Árbol limpio antes de crear este archivo.
- Proyecto Rust sencillo, un solo binario en `src/main.rs` (381 líneas en la revisión observada). `Cargo.toml` depende de `crossterm`, `serde_json` y `ureq`.
- Compilación documentada: `cargo build --release`; salida `target/release/xuper-cli`. No se compiló de nuevo al escribir esta memoria.
- `--url <catalog-url>` hace **GET** de un JSON o M3U; timeout global de 15 segundos y límite de cuerpo de 10 MiB. `--file <playlist.m3u>` carga un archivo local. Ambos imprimen el número de medios y hasta 20 títulos.
- `--search` abre una interfaz con `crossterm`: filtro por título mientras se escribe, flechas, Intro para reproducir, Esc/Ctrl-C para salir. Incluye títulos que solo tienen metadatos y los marca como no reproducibles.
- `--random` elige entre entradas que ya tienen una URL directa; usa el tiempo actual como índice, no un generador aleatorio fuerte.
- `--play <stream-url>` delega en `mpv`. Acepta esquemas HTTP, HTTPS, RTMP y RTSP. `XUPER_MPV_HEADLESS=1` usa `mpv` sin audio/vídeo y limita la lectura a 30 fotogramas. VLC no está implementado.
- `XUPER_AUTHORIZATION` se envía como cabecera `Authorization` **solo en la petición GET del catálogo**. No equivale al protocolo de autenticación de Xuper y no se aplica automáticamente a la reproducción.
- El parser JSON busca títulos en `title`, `name`, `vod_name`, `channel_name`; URLs directas en `stream_url`, `play_url`, `video_url`, `direct_source`, y a veces `url`, `src`, `file` si terminan en una extensión de vídeo reconocida. Recorre un conjunto fijo de claves anidadas y una profundidad máxima de ocho. No entiende todavía las respuestas reales de Xuper.
- `examples/demo.m3u` usa una transmisión pública de Big Buck Bunny para probar la interfaz. TVMaze, mencionado en `README.md`, aporta metadatos de series y **no** enlaces de reproducción de Xuper.
- `docs/` contiene archivos web estáticos heredados; no son el cliente Xuper funcional.
- Los APK versionados son `xuper/xuper.apk` (4.34.4) y `xuper/Xtv_4.99.20.apk` (4.99.20). La extracción `xuper/xuper/` está ignorada por Git y existe localmente. `target/` y `temp/` también están ignorados.

## APK y protección

| Archivo | Paquete / versión | SHA-256 del archivo |
| --- | --- | --- |
| `xuper/xuper.apk` | `com.android.mgstv`, 4.34.4, versionCode 43404 | `8a3b27b6b597910fce5ff63be193a9241441b58adad43e988cef557d1a26e9cb` |
| `xuper/Xtv_4.99.20.apk` | `com.android.mgstv`, 4.99.20, versionCode 49920 | `1d9f860b8630839ea7eb8ec969d7387f7ad32fc5f278858200144d2e89fcf1b1` |

- La APK 4.99.20 verifica con el certificado cuyo SHA-256 es `bd5ea57d9fc6f41677a0e97378e11d0aef2ba302c18716e88b9242fc3190857e`. La versión 5.2.0 descargada durante la investigación se verificó con el mismo certificado antes de instalarse **solo en un emulador**.
- Ambas APK están empaquetadas/protegidas. La 4.99.20 muestra clases `com.secneo.apkwrapper`, `DexHelper` y `dexdata0`: es una variante de SecNeo. Su `classes.dex` mide aproximadamente 30,4 MB; el DEX de arranque termina cerca del byte 1.883.940 y el resto contiene datos empaquetados. En 4.34.4, `classes.dex` es mucho menor y también se observaron bibliotecas de protección.
- JADX 1.5.6 aplicado a 4.99.20 solo expuso de forma útil el arranque y bibliotecas; gran parte de la lógica de negocio se carga/descifra en tiempo de ejecución. Una herramienta de desempaquetado `dxfx` para otra variante de SecNeo/DJI no reconoció la firma de esta APK. La presencia de cadenas de rutas en `dexdata0` **no revela** los cuerpos de solicitud, cabeceras, claves o respuestas.
- `assets/domain_test.json` en 4.99.20 contiene `"xx"` como valor para `portal_main`, `portal_backup`, `epg_main`, `market_main`, etc.; no aporta dominios utilizables.
- Se observó una configuración de red que permite tráfico HTTP en claro; no hay evidencia de que la app acepte una CA de usuario para interceptar HTTPS. La app del teléfono ignoró un proxy HTTP de Android durante la investigación. El enfoque con emulador instrumentado es más prometedor que cambiar el proxy del teléfono.
- Para reproducir los metadatos de las APK con las herramientas instaladas localmente: `/home/skellent/Android/Sdk/build-tools/36.0.0/aapt dump badging xuper/Xtv_4.99.20.apk`, `/home/skellent/Android/Sdk/build-tools/36.0.0/apksigner verify --print-certs xuper/Xtv_4.99.20.apk`, y `unzip -p xuper/Xtv_4.99.20.apk assets/domain_test.json`. No es necesario volver a descomprimir las APK en cada consulta.

## Observaciones de red: teléfono con 4.99.20 (2026-10-21)

La captura se hizo con PCAPdroid 2.0.2 filtrando el paquete `com.android.mgstv`. El teléfono era un TECNO_KI5k con Android 12. El archivo PCAP temporal medía unos 49 MB, contenía URLs firmadas y datos sensibles, y **no está en el repositorio ni seguía en `/tmp` al 2026-10-26**. Las siguientes son observaciones extraídas de aquella sesión, no una especificación completa del protocolo.

1. Durante reproducción VOD apareció una petición `POST` HTTP a `yuwc.swzablvpm.com/slb/v13/vod`. Su cuerpo y su respuesta eran datos codificados/opacos en la captura; no se obtuvo una estructura reutilizable.
2. Después se pidieron rutas `/vod/..._idx.idxg` y `/vod/..._media.ts` del mismo host. La URL `.ts` incluía parámetros firmados, entre ellos `token`, `sign2` y `expired`. Un pequeño rango descargado desde Linux devolvió datos MPEG-TS; `ffprobe` identificó vídeo HEVC y audio AAC. **No guardar ni reutilizar la URL completa**: contiene credenciales efímeras y vence.
3. Durante navegación del catálogo hubo conexiones HTTPS a `brkhzs.udigkkrc.com` y `tttopd.kj000zge.xyz`. El cifrado impedía ver el cuerpo de las peticiones de catálogo. No está probado que ambos hosts sirvan el mismo API, ni que sigan vigentes.
4. Un sondeo directo confirmó que `brkhzs.udigkkrc.com` acepta la ruta `api/portalCore/v3/searchByName`: varias variantes GET con parámetros `keyword`, `name`, `key` o `search` devolvieron HTTP 200 con cuerpo `null`; un POST JSON sencillo devolvió `{"returnCode":"500","errorMessage":"系统内部错误（未知错误）","data":null}`. `api/portalCore/v3/getShelveData` por GET devolvió el mismo error. Esto **no** valida el método, cuerpo, firma o cabeceras reales de la app.
5. Una prueba de ruta API sobre `tttopd.kj000zge.xyz` devolvió XML `AccessDenied`, compatible con un host CDN/almacenamiento, pero la función exacta de ese host es desconocida.

Los nombres de host pueden rotar. No los codifiques como fuente canónica del catálogo sin una captura reciente que muestre el host, la ruta, el formato de petición y la respuesta de la versión actual. No incrustes tokens, identificadores del teléfono, cookies, cabeceras privadas ni enlaces firmados en el repositorio.

## Rutas presentes como cadenas en `classes.dex` de 4.99.20

Son **cadenas estáticas**, no peticiones comprobadas. Este inventario completo se obtuvo con `unzip -p xuper/Xtv_4.99.20.apk classes.dex | strings -a -n 8` y una expresión regular para `api/portalCore/...`; hay 91 rutas distintas. Algunas versiones anteriores y posteriores de una misma operación aparecen juntas. La app puede no usar todas.

```text
api/portalCore/addSubscribe
api/portalCore/apkQueryCoupon
api/portalCore/apkReceiveCoupon
api/portalCore/bindEmailGiftDays
api/portalCore/bindPhone
api/portalCore/blSearchByContent
api/portalCore/blSearchByName
api/portalCore/changeBindPhone
api/portalCore/checkForceBind
api/portalCore/checkVerifiCode
api/portalCore/config/get
api/portalCore/delFavorite
api/portalCore/delSubscribe
api/portalCore/device
api/portalCore/device/updateOrInsert
api/portalCore/emailResetPassNotice
api/portalCore/epg/v2/getAllMatch
api/portalCore/epg/v2/getLineUps
api/portalCore/epg/v2/getMatchDetail
api/portalCore/epg/v2/getScoreboard
api/portalCore/epg/v2/getShelveMatch
api/portalCore/epg/v2/getTeamEvent
api/portalCore/epg/v3/getFootballMatch
api/portalCore/epg/v4/getNearestMatch
api/portalCore/epg/v5/getNearestMatch
api/portalCore/feedback/getCustomerService
api/portalCore/feedback/userFeedBack
api/portalCore/getAllbackProgram
api/portalCore/getAreaCode
api/portalCore/getBaseTime
api/portalCore/getEmailSuffix
api/portalCore/getExchangeOrderInfo
api/portalCore/getFavorite
api/portalCore/getHome
api/portalCore/getItemByPerson
api/portalCore/getNextColumns
api/portalCore/getPerson
api/portalCore/getPropertiesInfo
api/portalCore/getRecommendColumnContents
api/portalCore/getSubscribe
api/portalCore/getVerifiCode
api/portalCore/liveSearchByName
api/portalCore/package/getOrderInfo
api/portalCore/package/getPackageCustomization
api/portalCore/pwdCheck
api/portalCore/qr/allowLogin
api/portalCore/qr/token
api/portalCore/register
api/portalCore/searchResourceOrPerson
api/portalCore/setPassword
api/portalCore/terminalAuth
api/portalCore/unBindEmail
api/portalCore/updateBindEmailOrPwd
api/portalCore/v10/startPlayVOD
api/portalCore/v14/getSlbInfo
api/portalCore/v15/getSlbInfo
api/portalCore/v2/addFavorite
api/portalCore/v2/bindEmail
api/portalCore/v2/changeBindEmail
api/portalCore/v2/delFavorite
api/portalCore/v2/delSubscribe
api/portalCore/v2/getFree
api/portalCore/v2/getInviteCode
api/portalCore/v2/sendEmailVerifyCode
api/portalCore/v2/unBind
api/portalCore/v2/updateRestrictedStatus
api/portalCore/v2/validateVerifyCode
api/portalCore/v3/filterByContent
api/portalCore/v3/filterGenre
api/portalCore/v3/getColumnContents
api/portalCore/v3/getColumnRecommendResource
api/portalCore/v3/getProgram
api/portalCore/v3/getRecommends
api/portalCore/v3/getShelveData
api/portalCore/v3/searchByContent
api/portalCore/v3/searchByName
api/portalCore/v3/snToken
api/portalCore/v4/getItemData
api/portalCore/v4/resetPwd
api/portalCore/v4/startPlayLive
api/portalCore/v5/exchange
api/portalCore/v5/heartbeat
api/portalCore/v5/loginOut
api/portalCore/v5/qr/getResult
api/portalCore/v5/startPlayLive
api/portalCore/v6/getLiveData
api/portalCore/v6/startPlayBTV
api/portalCore/v7/getLiveData
api/portalCore/v8/active
api/portalCore/v8/login
api/portalCore/v9/getAuthInfo
```

Las rutas más interesantes para un MVP son `getHome`, `v3/getShelveData`, `v3/getColumnContents`, `v3/searchByName`, `v4/getItemData`, `v10/startPlayVOD` y `v15/getSlbInfo`. El orden concreto de llamada y las dependencias entre ellas siguen sin determinarse.

## Versión 5.2.0 y emulador instrumentado

- La app 4.99.20 instalada en un emulador mostró una actualización **obligatoria** a 5.2.0. El enlace mostrado por la app era `https://bit.ly/xtv520`, que en la sesión redirigió a `http://mgtpdw.eyikounzw.com/tp/Xtv_5.2.0.apk`. Estos enlaces son observaciones históricas, no una fuente permanente ni recomendación de descarga.
- El archivo descargado era `com.android.mgstv`, versionCode 50200, versionName 5.2.0, 36.255.373 bytes; se verificó que compartía certificado de firma con 4.99.20 y se instaló **solo en el emulador**, no en el teléfono real. `/tmp/Xtv_5.2.0.apk` ya no existía el 2026-10-26; no hay copia de 5.2.0 en el repositorio.
- Existe el AVD `XuperResearch` en `/home/skellent/.config/.android/avd/XuperResearch.avd`, configurado con la imagen `system-images/android-36.1/google_apis/x86_64/` y puente ARM64; `adb root` funcionó cuando se arrancó. Al 2026-10-26 no había dispositivos ADB conectados. El AVD conserva datos de usuario, pero **no se confirmó** que la instalación 5.2.0 siga intacta; comprobar antes de depender de ella.
- En aquella sesión, Frida 17.24.3 y un `frida-server` en el emulador permitieron adjuntarse al proceso principal de la app. En ejecución se enumeraron `okhttp3.OkHttpClient`, Retrofit y `com.main.ui.activity.HomeActivity`, lo que confirma que la lógica empaquetada se carga y que hay una ruta práctica para instrumentar las peticiones Java.
- Se preparó un hook a `okhttp3.OkHttpClient.newCall(okhttp3.Request)` que notificó estar listo. **No llegó a capturar una petición de búsqueda ni su respuesta**: la pantalla principal del emulador seguía cargando y faltó accionar la interfaz de búsqueda. Por tanto no se dispone del protocolo de catálogo de 5.2.0.
- En Frida 17, `-q` por sí solo hace que la CLI termine; para mantener el hook activo se usó `-q -t inf`. El intento de usar directamente `frida` desde Python falló con `Java is not defined` por no cargar el puente Java; la CLI sí lo incluía. Las clases de Okio estaban ofuscadas (`okio.a` a `okio.u`), así que no se pudo copiar el cuerpo de la petición buscando `okio.Buffer` por ese nombre.
- Los scripts, logs, JADX y Frida de esa sesión vivían en `/tmp` y ya no estaban disponibles el 2026-10-26. Si se repite la instrumentación, reconstruir los hooks; no asumir que los PID, scripts o logs antiguos siguen vivos. Capturar **método, host, ruta, nombres de cabeceras y esquema de cuerpo/respuesta** antes que valores privados. El siguiente paso de valor es identificar la búsqueda y `startPlayVOD` en tiempo de ejecución, después reproducir peticiones desde Linux.

## Incidente de proxy del teléfono y estado final comprobado

Durante la captura se dejó en el teléfono real un proxy global `127.0.0.1:8888`. Esto bloqueó Chrome, redes sociales y la validación de Internet de Android. Borrar `global_http_proxy_host` y `global_http_proxy_port` de Settings **no fue suficiente**: el monitor de red conservó el proxy en memoria y sus pruebas HTTPS seguían fallando con `Failed to connect to /127.0.0.1:8888`. Una prueba de `ping` o una conexión HTTP directa con `nc` no detecta ese fallo de proxy en las aplicaciones.

La corrección que funcionó el 2026-10-21 fue, por ADB, `settings put global http_proxy :0` y luego `svc wifi disable` seguido de `svc wifi enable`. El valor `:0` representó host vacío y puerto 0. Tras reconectar, Android informó `wifi_validated=True`; sus pruebas HTTP y HTTPS a `generate_204` respondieron 204. No se observó VPN activa; PCAPdroid estaba instalado, pero ProtonVPN era una aplicación propia del usuario y no se tocó. **No vuelvas a configurar un proxy global ni VPN de captura en el teléfono para esta investigación.** Si un futuro agente diagnostica la red, compruebe el estado actual antes de modificarlo, no asuma que sigue igual que en octubre.

## Próximos pasos técnicos

1. Verificar el estado actual del AVD y de la versión de la app; usar la versión que permita navegar. Mantener la ingeniería de red en el emulador con root/Frida, separado del teléfono real.
2. Capturar una búsqueda real y la carga de una ficha: método, host, ruta, nombres de cabeceras, tipo/estructura de cuerpo, código y tipo/estructura de respuesta. Guardar cualquier traza cruda fuera de Git y redactar antes de documentarla.
3. Determinar cómo obtiene la app el host dinámico y si un dispositivo invitado debe registrarse o firmar peticiones. Las rutas estáticas por sí solas no resuelven esto.
4. Capturar la transición `startPlayVOD` → `slb/v13/vod` → URL firmada → media. Separar datos del catálogo, metadatos de reproducción y parámetros efímeros. No usar una URL de captura como supuesto endpoint permanente.
5. Cuando exista una secuencia reproducible desde Linux, encapsularla en un módulo Rust, conectarla a la búsqueda actual y dejar `mpv` como reproductor inicial. Documentar qué depende de la versión y qué errores produce el servidor. Windows/VLC quedan para una fase posterior, conforme a la prioridad del usuario.

## Reglas prácticas para quien continúe

- Lee `README.md` y `src/main.rs` antes de modificar el cliente. Conserva el soporte JSON/M3U de demostración mientras incorporas Xuper.
- Distingue en notas y código entre **ruta en APK**, **petición observada**, **respuesta observada** y **protocolo reproducido**. No eleves una suposición a hecho.
- No agregues al repositorio PCAP, logs Frida sin redactar, cookies, claves, cabeceras de autorización, identificadores del teléfono ni URLs de media con firma. Los archivos de `/tmp` mencionados arriba ya no están y no forman parte del estado reproducible.
- Si hay emulador y teléfono conectados a la vez, usa `adb -s <serial>` explícitamente. Prefiere el emulador; no instales ni actualices nada en el teléfono real por inercia.
- La captura de red causó un problema real al usuario. Verifica la conectividad de Android mediante su **validación Wi-Fi y sondas HTTP/HTTPS**, no solo con `ping`, antes de dar por corregido cualquier cambio de red.
