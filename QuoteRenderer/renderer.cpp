#include "renderer.h"

#ifdef _DEBUG
#include <iostream>

namespace quote
{
    void renderer::render(const unsigned short& width, const unsigned short& height)
    {
        sf::RenderWindow window {sf::VideoMode(width, height), "Render debug window", \
            sf::Style::Default, sf::ContextSettings {0, 0, 8}};
        while (window.isOpen())
        {
            sf::Event e;
            while (window.pollEvent(e))
            {
                switch (e.type)
                {
                case sf::Event::Closed:
                    window.close();
                    break;
                default:
                    break;
                }
            }
    
            window.clear(sf::Color::Black);

            for (const auto shape: drawables_)
            {
                try
                {
                    window.draw(*shape);
                }
                catch (const std::exception& exception)
                {
                    std::cerr << exception.what() << "\n";
                }
            }
        
            window.display();
        }
    }

    void renderer::add_drawable(sf::Drawable* shape)
    {
        drawables_.push_back(shape);
    }
    

    void renderer::reserve(const unsigned int& count = 3)
    {
        drawables_.reserve(count);
    }



}

#else

#endif