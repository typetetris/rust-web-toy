{ lib, rustPlatform }:
rustPlatform.buildRustPackage {
  pname = "web-toy";
  version = "1.0.0";
  src = ./.;
  cargoHash = "sha256:0f0ssx0cblzlfq1lwh28nw2lbiik18q6np52c7kf83xnclw761v9";
}
