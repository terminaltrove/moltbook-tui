class MoltbookTui < Formula
  desc "TUI client for Moltbook, the social network for AI Agents"
  homepage "https://github.com/terminaltrove/moltbook-tui"
  version "1.0.0"
  license "MIT"

  on_macos do
    on_intel do
      url "https://github.com/terminaltrove/moltbook-tui/releases/download/v#{version}/moltbook-x86_64-apple-darwin.tar.gz"
      sha256 "0eac8d206dab599521b67fb171eef43748f3ae322cb826d43d373ce51d035253"
    end

    on_arm do
      url "https://github.com/terminaltrove/moltbook-tui/releases/download/v#{version}/moltbook-aarch64-apple-darwin.tar.gz"
      sha256 "c786b8faeb4c56ea89174ec43278e65a298632bef4f2dfa2c956bc318372824e"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/terminaltrove/moltbook-tui/releases/download/v#{version}/moltbook-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "11affe7b8788ed90a9971e8263bb59dca309090b8f54f103f5721a482a1db89d"
    end

    on_arm do
      url "https://github.com/terminaltrove/moltbook-tui/releases/download/v#{version}/moltbook-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "d3b2db76783304e6b8134824a89d70aecd5f9cf8fe49468e1e9d79d500caf7ee"
    end
  end

  def install
    bin.install "moltbook"
  end

  test do
    system "#{bin}/moltbook", "--version"
  end
end
