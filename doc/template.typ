#let setup(
  title: "TITLE",
  snippet: "SNIPPET",
  credits: "CREDITS",
  sidebar: "TITLE DESIGN DOCUMENT",
  notice: none,
  sidebar-color: aqua,
  body,
) = {
  set par(justify: true)
  set page(numbering: none)
  set text(font: "FreeSans")

  let numberingH(c) = {
    return numbering(c.numbering, ..counter(heading).at(c.location()))
  }

  let currentH(level: 1) = {
    let elems = query(selector(heading.where(level: level)).after(here()))

    if elems.len() != 0 and elems.first().location().page() == here().page() {
      return [#elems.first().body]
    } else {
      elems = query(selector(heading.where(level: level)).before(here()))
      if elems.len() != 0 {
        return [#elems.last().body]
      }
    }
    return ""
  }

  set page(
    header: context [
      #h(1fr)
      #counter(page).display(
        if page.numbering == none { "I" } else { "1/1" },
        both: page.numbering != none,
      )
    ],
    footer: rotate(-90deg, move(dx: 190pt, dy: -278pt, rect(fill: sidebar-color, height: 40pt, width: 850pt, align(
      center + horizon,
      text(fill: white, context [#currentH()]),
    )))),
    number-align: center,
  )

  hide[#heading(outlined: false, sidebar)]

  grid(
    rows: (1fr, 1fr),
    row-gutter: 20pt,
    box(width: 100%, height: 100%, align(bottom + center, [
      #text(font: "DejaVu Serif", size: 60pt, weight: "bold", title)
    ])),
    box(width: 100%, [
      #align(center, snippet)

      #align(center, credits)
    ]),
  )

  if notice != none {
    pagebreak()
    notice
  }

  pagebreak()
  heading(level: 2, outlined: false, "Outline")
  outline(indent: 10pt, title: none)
  set page(numbering: "1")
  counter(page).update(1)
  pagebreak()
  body
}
