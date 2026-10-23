(define username (get-payload 'username "Clanker Dev"))
(define user-role (get-payload 'user_role "user"))
(define user-status (get-payload 'user_status ""))

(define status-color
  (cond
    [(equal? user-role "creator")       (hex "#A682D1")]
    [(equal? user-role "administrator") (hex "#58AB63")]
    [else                               (hex "#21212166")]))

(define status-bg
  (cond
    [(equal? user-role "creator")       (hex "#A682D122")]
    [(equal? user-role "administrator") (hex "#58AB6322")]
    [else                               (hex "#00000000")]))

(define content (get-payload 'content "I'm stupid little dummy incapable of writing sowtware"))
(define avatar-initials (get-payload 'avatar_initials "CD"))
(define avatar-color-top (get-payload 'avatar_color_top "#ddd"))
(define avatar-color-bottom (get-payload 'avatar_color_bottom "#fff"))
(define name-color (get-payload 'name_color "#fff"))
(define avatar-image (get-payload 'image #f))
(define (has-image?) (if avatar-image #t #f))

(define theme-bg-color (hex "#2A2F33"))
(define theme-link-color (hex "#4B8FCA"))
(define theme-code-family "SF Mono")
(define theme-code-color (hex "#BFD4E7"))

(node (style (flex-row) (align-items 'end)
             (padding (px 2) (px 4) (px 4) (px 1)))

  (node (style (relative) (direction 'row) (align-items 'center) (justify-content 'center)
          (width (px 36)) (height (px 36))
          (margin-right (px 10)))
    (if (has-image?)
      (image avatar-image (circle (px 36)))
      (node (style (relative) (width (px 36)) (height (px 36))
              (direction 'row) (align-items 'center) (justify-content 'center))
        (node (style (absolute) (inset (px 0) (px 0) (px 0) (px 0)))
          (shape (circle)
            (fill (linear-gradient (hex avatar-color-top)
                    (hex avatar-color-bottom)))))
        (text avatar-initials
          (size (pt 14)) (color (hex "#ffffff")) (family "sans-serif") (bold)))))

  (node (style (direction 'column) (relative)
               (padding (px 10) (px 14) (px 14) (px 10))
               (max-width (px 400)))

    (node (style (absolute)
                 (inset (px 0) (px 0) (px 0) (px 0)))
      (shape (rounded-rect (px 16) (px 16) (px 16) (px 0))
             (fill (solid theme-bg-color))))

    ; From https://github.com/DrKLO/Telegram/blob/master/TMessagesProj/src/main/java/org/telegram/ui/ActionBar/MessageDrawable.java#L745
    (node (style (absolute) (left (px -5.708)) (bottom (px 0))
            (width (px 6.708)) (height (px 11)))
      (shape (svg-path "M 5.708 0 A 6.5 10 0 0 1 0.477 9.808 A 0.5 0.5 0 0 0 0.143 10.423 L 0.204 10.638 A 0.5 0.5 0 0 0 0.685 11 L 6.708 11 L 6.708 0 Z")
        (fill (solid theme-bg-color))))

    (node (style (flex-row) (align-items 'center) (justify-content 'space-between)
                 (margin-bottom (px 4)) (gap (px 6)))
      (text username
            (size (pt 15)) 
            (color (hex name-color))
            (family "sans-serif") (bold) (wrap #f))
      (if (not (equal? user-status ""))
        (node (style (relative) (direction 'row) (align-items 'center) (justify-content 'center)
                (padding-xy (px 6) (px 0)) (margin-right (px -6)))
          (node (style (absolute)
                  (inset (px 0) (px 0) (px 0) (px 0)))
            (shape (rounded-rect (px 100))
              (fill (solid status-bg))))
          (text user-status
            (size (pt 13)) (color status-color) (family "sans-serif") (wrap #f)))
        (node (style (hidden)))))

    (text content #:entities g-entities
      (size (pt 15))
      (overflow-wrap 'anywhere)
      (color (hex "#D8DFE5"))
      (family "SF Pro Display")
      (link-color theme-link-color)
      (code-color theme-code-color)
      (code-family theme-code-family)
    )
  )
)
