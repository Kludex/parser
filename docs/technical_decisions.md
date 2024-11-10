# Technical Decisions

Here are the technical decisions that were made during the development of the project.

## Ignore additional information before the first boundary delimiter

According to [RFC 2046#5.1.1](https://www.rfc-editor.org/rfc/rfc2046.html#section-5.1.1):

> There appears to be room for additional information prior to the first boundary
> delimiter line and following the final boundary delimiter line. These areas should
> generally be left blank, and implementations must ignore anything that appears
> before the first boundary delimiter line or after the last one.

For that, we also **ignore** any additional information **before the first boundary delimiter**.
