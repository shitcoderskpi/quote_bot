(define theme-bg-color     (hex "#EFFDDE"))
(define theme-link-color   (hex "#4CA635"))
(define theme-code-family  "SF Mono")
(define theme-code-color   (hex "#64AA7C"))

; From https://github.com/DrKLO/Telegram/blob/master/TMessagesProj/src/main/java/org/telegram/ui/ActionBar/MessageDrawable.java#L745
(define tail-path
  "M 5.708 0 A 6.5 10 0 0 1 0.477 9.808 A 0.5 0.5 0 0 0 0.143 10.423 L 0.204 10.638 A 0.5 0.5 0 0 0 0.685 11 L 6.708 11 L 6.708 0 Z")

(define (status-color role)
  (cond
    [(equal? role "creator")       (hex "#955CDB")]
    [(equal? role "administrator") (hex "#40A920")]
    [else                          (hex "#999999")]))

(define (status-bg role)
  (cond
    [(equal? role "creator")       (hex "#955CDB1F")]
    [(equal? role "administrator") (hex "#40A9201F")]
    [else                          (hex "#00000000")]))

(define (avatar m)
  (define img (get-payload m 'image #f))
  (node (style (relative) (direction 'row) (align-items 'center) (justify-content 'center)
          (width (px 36)) (height (px 36))
          (margin-right (px 10)))
    (if img
      (image img (circle (px 36)))
      (node (style (relative) (width (px 36)) (height (px 36))
              (direction 'row) (align-items 'center) (justify-content 'center))
        (node (style (absolute) (inset (px 0) (px 0) (px 0) (px 0)))
          (shape (circle)
            (fill (linear-gradient
                    (hex (get-payload m 'avatar_color_top "#ddd"))
                    (hex (get-payload m 'avatar_color_bottom "#fff"))))))
        (text (get-payload m 'avatar_initials "CD")
          (size (pt 14)) (color (hex "#ffffff")) (family "sans-serif") (bold))))))

;; keeps bubbles aligned when the avatar is not shown
(define (avatar-spacer)
  (node (style (width (px 36)) (margin-right (px 10)))))

(define (header m)
  (define user-status (get-payload m 'user_status ""))
  (define user-role   (get-payload m 'user_role "user"))
  (node (style (flex-row) (align-items 'center) (justify-content 'space-between)
          (margin-bottom (px 4)) (gap (px 6)))
    (text (get-payload m 'username "Clanker Dev")
      (size (pt 15))
      (color (hex (get-payload m 'name_color "#fff")))
      (family "sans-serif") (bold) (wrap #f))
    (if (equal? user-status "")
      #f
      (node (style (relative) (direction 'row) (align-items 'center) (justify-content 'center)
              (padding-xy (px 6) (px 0)) (margin-right (px -6)))
        (node (style (absolute) (inset (px 0) (px 0) (px 0) (px 0)))
          (shape (rounded-rect (px 100))
            (fill (solid (status-bg user-role)))))
        (text user-status
          (size (pt 13)) (color (status-color user-role))
          (family "sans-serif") (wrap #f))))))

(define (render-message m)
  (define start? (get-payload m 'group_start #t))
  (define end?   (get-payload m 'group_end #t))

  (node (style (flex-row) (align-items 'end)
          (padding (px 2) (px 4) (if end? (px 4) (px 0)) (px 1)))

    (if end? (avatar m) (avatar-spacer))

    (node (style (direction 'column) (relative)
            (padding (px 10) (px 14) (px 14) (px 10))
            (max-width (px 400)))

      (node (style (absolute) (inset (px 0) (px 0) (px 0) (px 0)))
        (shape (rounded-rect (if start? (px 16) (px 6))
                 (px 16)
                 (px 16)
                 (if end? (px 0) (px 6)))
          (fill (solid theme-bg-color))))

      (if end?
        (node (style (absolute) (left (px -5.508)) (bottom (px 0))
                (width (px 6.708)) (height (px 11)))
          (shape (svg-path tail-path)
            (fill (solid theme-bg-color))))
        #f)

      (if start? (header m) #f)

      (text (get-payload m 'content "")
        #:entities (get-payload m 'entities #f)
        (size (pt 15))
        (color (hex "#000000"))
        (family "SF Pro Display")
        (overflow-wrap 'anywhere)
        (link-color theme-link-color)
        (code-color theme-code-color)
        (code-family theme-code-family)))))

(%make-node
  (cons (style (flex-column) (align-items 'start))
    (map render-message (hash-ref payload 'messages))))