(define username (get-payload 'username "Unknown User"))
(define user-role (get-payload 'user_role "user"))
(define raw-status (get-payload 'user_status ""))
(define user-status (if (equal? raw-status "") "" (string-append " " raw-status)))

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

(define content (string-append (get-payload 'content "") "   "))
(define avatar-initials (get-payload 'avatar_initials ""))
(define avatar-top (get-payload 'avatar_color_top "#ff0000"))
(define avatar-bottom (get-payload 'avatar_color_bottom "#dd0000"))
(define has-image? (not (equal? (get-payload 'image "") "")))

(define theme-link-color (hex "#4B8FCA"))
(define theme-code-family "monospace")

(node (style (flex-row) (align-items 'end)
             (padding (px 2) (px 4) (px 4) (px 1)))

  (node (style (direction 'row) (align-items 'center) (justify-content 'center)
               (width (px 36)) (height (px 36))
               (margin-right (px 10)))
    (if has-image?
        (node (style (hidden)))
        (node (style (absolute)
                     (inset (px 0) (px 0) (px 0) (px 0)))
          (shape (circle)
                 (fill (linear-gradient (hex avatar-top) (hex avatar-bottom))))))
    (if has-image?
        (image (get-payload 'image "") (circle (px 36)))
        (text avatar-initials
              (size (pt 14)) (color (hex "#ffffff")) (family "sans-serif") (weight 700))))

  (node (style (direction 'column) (relative)
               (padding (px 10) (px 14) (px 14) (px 10))
               (max-width (px 500)))

    (node (style (absolute)
                 (inset (px 0) (px 0) (px 0) (px 0)))
      (shape (rounded-rect (px 16) (px 16) (px 16) (px 0))
             (fill (solid (hex "#2A2F33")))))
             
    (node (style (absolute) (left (px -10)) (bottom (px 0))
                 (width (px 11)) (height (px 18)))
      (shape (svg-path "M 11 0 Q 11 18 0 18 L 11 18 Z")
             (fill (solid (hex "#2A2F33")))))

    (node (style (flex-row) (align-items 'center) (justify-content 'space-between)
                 (margin-bottom (px 4)) (gap (px 6)))
      (text username
            (size (pt 15)) (color (hex "#4D9AD9")) (family "sans-serif") (weight 700))
      (if (not (equal? raw-status ""))
          (node (style (direction 'row) (align-items 'center) (justify-content 'center)
                       (padding-xy (px 6) (px 0)) (margin-right (px -6)))
            (node (style (absolute)
                         (inset (px 0) (px 0) (px 0) (px 0)))
              (shape (rounded-rect (px 100))
                     (fill (solid status-bg))))
            (text raw-status
                  (size (pt 13)) (color status-color) (family "sans-serif")))
          (node (style (hidden)))))

    (text content
          (size (pt 15)) (color (hex "#D8DFE5")) (family "SF Pro Display")
          (link-color theme-link-color)
          (code-family theme-code-family))))
