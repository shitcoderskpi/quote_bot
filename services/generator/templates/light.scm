;; light.scm — Quote bubble template (light theme)

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

(define content (string-append (get-payload 'content "No content provided") "   "))
(define avatar-initials (get-payload 'avatar_initials ""))
(define avatar-top (get-payload 'avatar_color_top "#ffffff"))
(define avatar-bottom (get-payload 'avatar_color_bottom "#dddddd"))
(define has-image? (not (equal? (get-payload 'image "") "")))

;; Global style settings
(define theme-link-color (hex "#4CA635"))
(define theme-code-family "monospace")

;; Root: flex row, avatar + bubble
(node (style (flex-row) (align-items 'end)
             (padding (px 2) (px 4) (px 4) (px 1)))

  ;; 1. Avatar Container
  (node (style (direction 'row) (align-items 'center) (justify-content 'center)
               (width (px 36)) (height (px 36))
               (margin-right (px 10)))
    ;; Avatar background circle
    (node (style (absolute)
                 (inset (px 0) (px 0) (px 0) (px 0)))
      (shape (circle)
             (fill (linear-gradient (hex avatar-top) (hex avatar-bottom)))))
    ;; Avatar initials or image
    (if has-image?
        (image (get-payload 'image "") (circle))
        (text avatar-initials
              (size (pt 14)) (color (hex "#ffffff")) (family "sans-serif") (weight 700))))

  ;; 2. Bubble Container
  (node (style (direction 'column) (relative)
               (padding (px 10) (px 14) (px 14) (px 10))
               (max-width (px 500)))

    ;; 2a. Background bubble
    (node (style (absolute)
                 (inset (px 0) (px 0) (px 0) (px 0)))
      (shape (rounded-rect (px 16) (px 16) (px 16) (px 0))
             (fill (solid (hex "#EFFDDE")))))

    ;; 2b. Bubble tail
    (node (style (absolute) (left (px -10)) (bottom (px 0))
                 (width (px 10)) (height (px 18)))
      (shape (svg-path "M 10 0 Q 10 18 0 18 L 10 18 Z")
             (fill (solid (hex "#EFFDDE")))))

    ;; 2c. Username + Status row
    (node (style (flex-row) (align-items 'center) (justify-content 'space-between)
                 (margin-bottom (px 4)) (gap (px 6)))
      ;; Username
      (text username
            (size (pt 15)) (color (hex "#4CA635")) (family "sans-serif") (weight 700))
      ;; Status tag
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

    ;; 2d. Message content
    (text content
          (size (pt 15)) (color (hex "#000000")) (family "sans-serif")
          (link-color theme-link-color)
          (code-family theme-code-family))))
