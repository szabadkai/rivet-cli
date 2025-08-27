class Rivet < Formula
  desc "API testing that lives in git"
  homepage "https://github.com/szabadkai/rivet-cli"
  url "https://github.com/szabadkai/rivet-cli/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_SHA256"
  license "MIT"
  head "https://github.com/szabadkai/rivet-cli.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/rivet --version")
  end
end

