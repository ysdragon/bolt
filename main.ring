// The Main File
load "package.ring"
load "lib.ring"
load "src/utils/color.ring"

func main() {
    nInnerWidth = 51
    cHLine = copy("─", nInnerWidth)

    banner = []

    // Top border
    banner + colorText([:text = "╭" + cHLine + "╮", :color = :BRIGHT_YELLOW, :style = :BOLD])
    banner + colorText([:text = "│" + space(nInnerWidth) + "│", :color = :BRIGHT_YELLOW])

    // Title with volt symbol
    cName = upper(aPackageInfo[:name])
    cTitleText = "⚡ " + cName + " ⚡"
    nTitleVisualWidth = len(cTitleText) - 2
    nTitlePad = floor((nInnerWidth - nTitleVisualWidth) / 2)
    nTitlePadRight = nInnerWidth - nTitleVisualWidth - nTitlePad
    cTitleLine = colorText([:text = "│" + space(nTitlePad), :color = :BRIGHT_YELLOW]) +
                 colorText([:text = cTitleText, :color = :BRIGHT_YELLOW, :style = :BOLD]) +
                 colorText([:text = space(nTitlePadRight) + "│", :color = :BRIGHT_YELLOW])
    banner + cTitleLine

    banner + colorText([:text = "│" + space(nInnerWidth) + "│", :color = :BRIGHT_YELLOW])

    // Version
    cVersionStr = "v" + aPackageInfo[:version]
    nVersionPad = floor((nInnerWidth - len(cVersionStr)) / 2)
    nVersionPadRight = nInnerWidth - len(cVersionStr) - nVersionPad
    cVersionLine = colorText([:text = "│" + space(nVersionPad), :color = :BRIGHT_YELLOW]) +
                   colorText([:text = cVersionStr, :color = :WHITE, :style = :DIM]) +
                   colorText([:text = space(nVersionPadRight) + "│", :color = :BRIGHT_YELLOW])
    banner + cVersionLine

    banner + colorText([:text = "│" + space(nInnerWidth) + "│", :color = :BRIGHT_YELLOW])

    // Separator
    nDotsCount = nInnerWidth - 12
    cSepLine = colorText([:text = "│" + space(6), :color = :BRIGHT_YELLOW]) +
               colorText([:text = copy("·", nDotsCount), :color = :WHITE, :style = :DIM]) +
               colorText([:text = space(6) + "│", :color = :BRIGHT_YELLOW])
    banner + cSepLine

    banner + colorText([:text = "│" + space(nInnerWidth) + "│", :color = :BRIGHT_YELLOW])

    // Description
    cDesc = "Blazing-fast web framework for Ring"
    nDescPad = floor((nInnerWidth - len(cDesc)) / 2)
    nDescPadRight = nInnerWidth - len(cDesc) - nDescPad
    cDescLine = colorText([:text = "│" + space(nDescPad), :color = :BRIGHT_YELLOW]) +
                colorText([:text = cDesc, :color = :WHITE, :style = :DIM]) +
                colorText([:text = space(nDescPadRight) + "│", :color = :BRIGHT_YELLOW])
    banner + cDescLine

    banner + colorText([:text = "│" + space(nInnerWidth) + "│", :color = :BRIGHT_YELLOW])

    // Separator
    banner + cSepLine

    banner + colorText([:text = "│" + space(nInnerWidth) + "│", :color = :BRIGHT_YELLOW])

    // Author
    cAuthorText = "Made with  by " + aPackageInfo[:developer]
    nAuthorVisualWidth = len(cAuthorText) + 1
    nAuthorPad = floor((nInnerWidth - nAuthorVisualWidth) / 2)
    nAuthorPadRight = nInnerWidth - nAuthorVisualWidth - nAuthorPad
    cAuthorLine = colorText([:text = "│" + space(nAuthorPad), :color = :BRIGHT_YELLOW]) +
                  colorText([:text = "Made with ", :color = :WHITE, :style = :DIM]) +
                  colorText([:text = cSymbols[:HEART], :color = :BRIGHT_RED]) +
                  colorText([:text = " by ", :color = :WHITE, :style = :DIM]) +
                  colorText([:text = aPackageInfo[:developer], :color = :BRIGHT_CYAN]) +
                  colorText([:text = space(nAuthorPadRight) + "│", :color = :BRIGHT_YELLOW])
    banner + cAuthorLine

    banner + colorText([:text = "│" + space(nInnerWidth) + "│", :color = :BRIGHT_YELLOW])

    // GitHub URL
    cUrlStr = aPackageInfo[:providerwebsite] + "/" + aPackageInfo[:providerusername] + "/" + aPackageInfo[:folder]
    nUrlPad = floor((nInnerWidth - len(cUrlStr)) / 2)
    nUrlPadRight = nInnerWidth - len(cUrlStr) - nUrlPad
    cUrlLine = colorText([:text = "│" + space(nUrlPad), :color = :BRIGHT_YELLOW]) +
               colorText([:text = cUrlStr, :color = :GREEN, :style = :UNDERLINE]) +
               colorText([:text = space(nUrlPadRight) + "│", :color = :BRIGHT_YELLOW])
    banner + cUrlLine

    banner + colorText([:text = "│" + space(nInnerWidth) + "│", :color = :BRIGHT_YELLOW])

    // Bottom border
    banner + colorText([:text = "╰" + cHLine + "╯", :color = :BRIGHT_YELLOW, :style = :BOLD])

    // Print banner
    ? ""
    for line in banner {
        ? "  " + line
    }
    ? ""
}