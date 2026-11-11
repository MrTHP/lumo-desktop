# Bonsai unload integration

Model Desk's **Unload model** button and its automatic unload-on-quit call
`POST /unload` on the Bonsai backend. Upstream
[Bonsai-Image-Demo](https://github.com/PrismML-Eng/Bonsai-Image-Demo) does not
expose that endpoint, so Model Desk ships this small patch for the Linux/CUDA
`scripts/local_backend.py` shim.

The patch adds:

- `GET /memory` — reports whether the model is loaded and how much GPU memory
  PyTorch currently holds.
- `POST /unload` — drops the transformer, text encoder, VAE and scheduler,
  empties the CUDA cache, and reports the freed megabytes.
- Transparent reload: the next `/generate` request calls `prewarm()` again.
- `PYTORCH_CUDA_ALLOC_CONF=expandable_segments:True` to keep unload/reload
  cycles stable on 8 GB GPUs.

## Apply

```sh
cd /path/to/Bonsai-Image-Demo
git apply /path/to/model-desk/integrations/bonsai-unload.patch
# restart the backend
./scripts/serve.sh
```

Check it:

```sh
curl http://127.0.0.1:8000/memory
curl -X POST http://127.0.0.1:8000/unload
```

If the patch does not apply because your Bonsai checkout changed, the app still
works for image generation; the unload button is simply hidden when `/memory`
is unavailable. The macOS/MLX shim (`scripts/local_backend_mac.py`) is not
covered by this patch.
