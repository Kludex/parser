# Technical Decisions

Here are the technical decisions that were made during the development of the project.

## Ignore additional information before the first boundary delimiter

According to [RFC 2046#5.1.1](https://www.rfc-editor.org/rfc/rfc2046.html#section-5.1.1):

> There appears to be room for additional information prior to the first boundary
> delimiter line and following the final boundary delimiter line. These areas should
> generally be left blank, and implementations must ignore anything that appears
> before the first boundary delimiter line or after the last one.

For that, we also **ignore** any additional information **before the first boundary delimiter**.

## Ignore `Content-Transfer-Encoding` header

As per [RFC 7578#4.7](https://www.rfc-editor.org/rfc/rfc7578.html#section-4.7):

> [...] Senders SHOULD NOT generate any parts with a Content-Transfer-Encoding header field.

For that, we also **ignore** the `Content-Transfer-Encoding` header.
