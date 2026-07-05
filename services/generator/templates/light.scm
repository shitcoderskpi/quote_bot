;; light.scm
;; Declarative UI template for the quote bot using Steel Scheme!

(define (get-payload key default-val)
  (let ((found (assoc key payload)))
    (if (and found (not (null? (cdr found))))
        (cdr found)
        default-val)))

(define username (get-payload 'username "Unknown User"))
(define user-role (get-payload 'user_role "user"))
(define raw-status (get-payload 'user_status ""))
(define user-status (if (equal? raw-status "") "" (string-append " " raw-status)))

(define status-color
  (if (equal? user-role "creator") "#A682D1"
      (if (equal? user-role "administrator") "#58AB63"
          "#21212166")))

(define status-bg
  (if (equal? user-role "creator") "#A682D122"
      (if (equal? user-role "administrator") "#58AB6322"
          "#00000000")))

(define content (string-append (get-payload 'content "No content provided") "   ")) ;; pad right
(define avatar-initials (get-payload 'avatar_initials ""))
(define avatar-top (get-payload 'avatar_color_top "#ffffff"))
(define avatar-bottom (get-payload 'avatar_color_bottom "#dddddd"))
(define has-image? (not (equal? (get-payload 'image #f) #f)))

;; The final layout is a flex row containing the avatar and the bubble.
(hash
  'style (hash 
    'display 'flex
    'flex_direction 'row
    'align_items 'flex-end
    'padding (hash 'top (list 'px 2) 'bottom (list 'px 4) 'left (list 'px 1) 'right (list 'px 4))
  )
  'content (hash
    'group (list
      
      ;; 1. The Avatar Container
      (hash
        'style (hash
          'display 'flex
          'align_items 'center
          'justify_content 'center
          'size (hash 'width (list 'px 36) 'height (list 'px 36))
          'margin (hash 'right (list 'px 10))
        )
        'content (hash
          'group (list
            ;; Avatar Background
            (hash
              'style (hash
                'position (hash 'type 'absolute 'top (list 'px 0) 'left (list 'px 0) 'right (list 'px 0) 'bottom (list 'px 0))
              )
              'content (hash
                'shape (hash
                  'kind (hash 'circle (hash 'radius 'auto))
                  'fill (list 'linear-gradient (list 'hex avatar-top) (list 'hex avatar-bottom))
                )
              )
            )
            ;; Avatar Initials
            (if (not has-image?)
                (hash
                  'style (hash)
                  'content (hash
                    'text (hash 'value avatar-initials 'size 14 'color (list 'hex "#ffffff") 'family "sans-serif" 'weight 700)
                  )
                )
                (hash 'style (hash 'display 'none) 'content (hash 'group (list)))
            )
          )
        )
      )

      ;; 2. The Bubble Container
      (hash
        'style (hash
          'display 'flex
          'flex_direction 'column
          'position (hash 'type 'relative)
          'padding (hash 'top (list 'px 10) 'right (list 'px 14) 'bottom (list 'px 14) 'left (list 'px 10))
          'max_size (hash 'width (list 'px 500))
        )
        'content (hash
          'group (list
            
            ;; 2a. The Background Bubble
            (hash
              'style (hash
                'position (hash 'type 'absolute 'top (list 'px 0) 'left (list 'px 0) 'right (list 'px 0) 'bottom (list 'px 0))
              )
              'content (hash
                'shape (hash
                  'kind (hash 'rect (hash 'corners (hash 'top_left (list 'px 16) 'top_right (list 'px 16) 'bottom_right (list 'px 16) 'bottom_left (list 'px 0))))
                  'fill (list 'solid (list 'hex "#EFFDDE"))
                )
              )
            )

            ;; 2b. The Bubble "Tail"
            (hash
              'style (hash
                'position (hash 'type 'absolute 'left (list 'px -10) 'bottom (list 'px 0))
                'size (hash 'width (list 'px 10) 'height (list 'px 18))
              )
              'content (hash
                'shape (hash
                  'kind (hash 'path (hash 'data "M 10 0 Q 10 18 0 18 L 10 18 Z"))
                  'fill (list 'solid (list 'hex "#EFFDDE"))
                )
              )
            )

            ;; 2c. Username + Status Container
            (hash
               'style (hash 
                 'display 'flex
                 'flex_direction 'row
                 'align_items 'center
                 'justify_content 'space-between
                 'margin (hash 'bottom (list 'px 4))
                 'gap (hash 'x (list 'px 6))
               )
               'content (hash 'group (list
                 ;; Username
                 (hash
                   'style (hash)
                   'content (hash 'text (hash 'value username 'size 15 'color (list 'hex "#4CA635") 'family "sans-serif" 'weight 700))
                 )
                 ;; Status Tag
                 (if (not (equal? raw-status ""))
                     (hash
                       'style (hash
                         'display 'flex
                         'align_items 'center
                         'justify_content 'center
                         'padding (hash 'top (list 'px 0) 'bottom (list 'px 0) 'left (list 'px 6) 'right (list 'px 6))
                         'margin (hash 'right (list 'px -6))
                       )
                       'content (hash
                         'group (list
                           ;; Status Bg
                           (hash
                             'style (hash 'position (hash 'type 'absolute 'top (list 'px 0) 'left (list 'px 0) 'right (list 'px 0) 'bottom (list 'px 0)))
                             'content (hash 'shape (hash 'kind (hash 'rect (hash 'corners (hash 'all (list 'px 100))))
                                                         'fill (list 'solid (list 'hex status-bg))))
                           )
                           ;; Status Text
                           (hash
                             'style (hash)
                             'content (hash 'text (hash 'value raw-status 'size 13 'color (list 'hex status-color) 'family "sans-serif" 'weight 400))
                           )
                         )
                       )
                     )
                     (hash 'style (hash 'display 'none) 'content (hash 'group (list)))
                 )
               ))
            )
            
            ;; 2d. Message Content
            (hash
               'style (hash)
               'content (hash 'text (hash 'value content 'size 15 'color (list 'hex "#000000") 'family "sans-serif"))
            )

          )
        )
      )
    )
  )
)
