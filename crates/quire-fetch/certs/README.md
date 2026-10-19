# Intermediate certificates for institutional proxies

Some proxies send only their own certificate, without the intermediate that links it to a root
the system trusts. Browsers download the missing intermediate (from the certificate's "CA
Issuers" address); Rust's TLS stack does not, so quire carries the ones users need. They are
added to the proxy client only, and still have to chain to a system root.

| File | For | Issuer (root) | Source | SHA-256 |
|---|---|---|---|---|
| `incommon-rsa-ov-ssl-ca-3.pem` | `*.lp.hscl.ufl.edu` (University of Florida) and other InCommon members | Sectigo Public Server Authentication Root R46 | http://crt.sectigo.com/InCommonRSAOVSSLCA3.crt | `9C:0F:28:8E:AA:BB:71:40:57:01:02:0C:97:2F:81:59:44:62:96:07:8D:38:59:1C:68:49:9B:12:CE:37:94:44` |

Checked 2026-10-19 with `openssl verify` against the system bundle (OK) and `openssl s_client`
to `login.lp.hscl.ufl.edu` with it (verify return code 0). Other institutions: `quire config set
access.ca <file.pem>`.
