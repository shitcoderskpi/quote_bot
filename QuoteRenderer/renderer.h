#pragma once
#include <SFML/Graphics.hpp>

namespace quote
{
    class renderer;
}


namespace quote
{
#ifdef _DEBUG

#pragma comment(lib, "sfml-system-d.lib")
#pragma comment(lib, "sfml-graphics-d.lib")
#pragma comment(lib, "sfml-window-d.lib")
    class renderer
    {

        std::vector<sf::Drawable*> drawables_;
        
    public:
        explicit renderer() noexcept = default;

        void add_drawable(sf::Drawable* shape);
        void reserve(const unsigned int& count);

        void render(const unsigned short& width, const unsigned short& height);
    };

#else

    class renderer
    {
    public:
        
    };
    
#endif
}
