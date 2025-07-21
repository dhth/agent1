agent1
===

My first attempt at building an AI agent.

To minimize abstractions over the underlying models, `agent1` doesn't use client
libraries from LLM providers, instead, it communicates directly with their REST
endpoints.

> [!WARNING]
> This project is for educational purposes only. It is not intended for
> production use.

Tools available
---

`agent1` has the following tools available:

- `read_file`
- `list_files`
- `edit_file`
- `run_command`
