class Rivet < Formula
  desc "API testing that lives in git"
  homepage "https://github.com/__REPO__"
  version "__VERSION_STRIP_V__"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "__BASE_URL__/rivet-__VERSION__-macos-arm64.tar.gz"
      sha256 "__MAC_ARM_SHA__"
    else
      url "__BASE_URL__/rivet-__VERSION__-macos-x86_64.tar.gz"
      sha256 "__MAC_INTEL_SHA__"
    end
  end

  on_linux do
    url "__BASE_URL__/rivet-__VERSION__-linux-x86_64.tar.gz"
    sha256 "__LINUX_SHA__"
  end

  def install
    bin.install "rivet"
    (bash_completion/"rivet").write Utils.safe_popen_read(bin/"rivet", "completions", "bash")
    (zsh_completion/"_rivet").write Utils.safe_popen_read(bin/"rivet", "completions", "zsh")
    (fish_completion/"rivet.fish").write Utils.safe_popen_read(bin/"rivet", "completions", "fish")
    (man1/"rivet.1").write Utils.safe_popen_read(bin/"rivet", "man")
  end

  test do
    system "#{bin}/rivet", "--version"
  end
end

